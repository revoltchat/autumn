pub mod config;
pub mod db;
pub mod routes;
pub mod util;
pub mod version;

use futures::StreamExt;
use util::variables::{CONFIG, HOST, LOCAL_STORAGE_PATH, USE_S3};

#[macro_use]
extern crate lazy_static;
extern crate tree_magic;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use log::info;
use mongodb::bson::doc;
use std::env;

pub static CACHE_CONTROL: &str = "public, max-age=604800, must-revalidate";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    if let Ok(v) = env::var("MINIO_ROOT_USER") {
        env::set_var("AWS_ACCESS_KEY_ID", v);
    }

    if let Ok(v) = env::var("MINIO_ROOT_PASSWORD") {
        env::set_var("AWS_SECRET_ACCESS_KEY", v);
    }

    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    config::Config::init()
        .unwrap_or_else(|err| panic!("Unable to load the config '{}'. {}", *CONFIG, err));

    info!("Starting Autumn server.");

    db::connect().await;

    if !*USE_S3 {
        info!("Ensuring local storage directory exists.");
        std::fs::create_dir_all(LOCAL_STORAGE_PATH.to_string()).unwrap();
    } else {
        info!("Skipping existence check, make sure your S3 buckets exist!");
    }

    tokio::spawn(async {
        let mut sched = tokio_cron_scheduler::JobScheduler::new();

        sched
            .add(
                tokio_cron_scheduler::Job::new_repeated(
                    core::time::Duration::from_secs(600),
                    |_, _| {
                        tokio::spawn(async {
                            let col = db::get_collection("attachments");
                            let mut cursor = col
                                .find(
                                    doc! {
                                        "deleted": true,
                                        "reported": {
                                            "$ne": true
                                        }
                                    },
                                    None,
                                )
                                .await
                                .unwrap();

                            while let Some(result) = cursor.next().await {
                                if let Ok(file) = result {
                                    file.delete().await.unwrap();
                                }

                                // Delay before doing the next item in list.
                                tokio::time::sleep(core::time::Duration::from_millis(50)).await;
                            }
                        });
                    },
                )
                .unwrap(),
            )
            .unwrap();

        sched.start().await.unwrap();
    });

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|_, _| true)
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(["X-Session-Token", "X-Bot-Token"])
                    .supports_credentials(),
            )
            .wrap(middleware::Logger::default())
            .route("/{tag:[^/]*}", web::post().to(routes::upload::post))
            .route(
                "/{tag:[^/]*}/download/{filename:.*}",
                web::get().to(routes::download::get),
            )
            .route(
                "/{tag:[^/]*}/{filename:[^/]*}",
                web::get().to(routes::serve::get),
            )
            .route(
                "/{tag:[^/]*}/{filename:[^/]*}/{fn:.*}",
                web::get().to(routes::serve::get),
            )
            .route("/", web::get().to(routes::index::get))
    })
    .bind(HOST.clone())?
    .run()
    .await
}

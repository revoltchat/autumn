pub mod config;
pub mod db;
pub mod routes;
pub mod util;
pub mod version;

use util::variables::{HOST, LOCAL_STORAGE_PATH, USE_S3};

#[macro_use]
extern crate lazy_static;
extern crate tree_magic;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use log::info;
use std::env;

pub static CACHE_CONTROL: &'static str = "public, max-age=604800, must-revalidate";

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
    config::Config::init()?;

    info!("Starting Autumn server.");

    db::connect().await;

    if !*USE_S3 {
        info!("Ensuring local storage directory exists.");
        std::fs::create_dir_all(LOCAL_STORAGE_PATH.to_string()).unwrap();
    } else {
        info!("Skipping existence check, make sure your S3 buckets exist!");
    }

    tokio::spawn(async {
        // loop {
            // delete
        // }
    });

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|_, _| true)
                    .allowed_methods(vec!["GET", "POST"])
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

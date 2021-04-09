pub mod db;
pub mod util;
pub mod config;
pub mod routes;

use util::variables::{HOST, USE_S3, LOCAL_STORAGE_PATH};

#[macro_use]
extern crate lazy_static;
extern crate tree_magic;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));
    config::Config::init()?;

    info!("Starting Autumn server.");

    db::connect().await;

    if !*USE_S3 {
        info!("Ensuring local storage directory exists.");
        std::fs::create_dir_all(LOCAL_STORAGE_PATH.to_string()).unwrap();
    }

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|_, _| true)
                    .allowed_methods(vec!["GET", "POST"])
                    .supports_credentials(),
            )
            .wrap(middleware::Logger::default())
            .route(
                "/{tag:[^/]*}",
                web::post().to(routes::upload::post),
            )
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
            .route(
                "/",
                web::get().to(routes::index::get),
            )
    })
    .bind(HOST.clone())?
    .run()
    .await
}

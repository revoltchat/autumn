pub mod db;
pub mod routes;
pub mod util;

use util::variables::HOST;

#[macro_use]
extern crate lazy_static;
extern crate tree_magic;

use actix_web::{middleware, web, App, HttpServer};
use log::info;
use web::PayloadConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!("Starting Autumn server.");

    db::connect().await;
    std::fs::create_dir_all("./files").unwrap();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(PayloadConfig::new(10_000_000))
            .service(
                web::resource("/")
                    // .route(web::get().to(index))
                    .route(web::post().to(routes::upload::upload)),
            )
            .route(
                "/{filename:[^/]*}",
                web::get().to(routes::static_serve::static_serve),
            )
            .route(
                "/{filename:[^/]*}/{fn:.*}",
                web::get().to(routes::static_serve::static_serve),
            )
    })
    .bind(HOST.clone())?
    .run()
    .await
}

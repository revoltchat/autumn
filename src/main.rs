use std::{fs::File, io::Write};

use actix_multipart::Multipart;
use actix_web::{App, Error, FromRequest, HttpResponse, HttpServer, middleware, web};
use futures::{Stream, StreamExt, TryStreamExt};

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    if let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let fpath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));

        let mut f = {
            let fpath = fpath.clone();
            web::block(|| std::fs::File::create(fpath))
                .await
                .unwrap()
        };
        
        let delete = |f: File| async {
            drop(f);
            let fpath = fpath.clone();
            web::block(|| std::fs::remove_file(fpath))
                .await
                .unwrap();
            
            Ok(HttpResponse::BadRequest().into())
        };

        let mut file_size: usize = 0;
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_size += data.len();

            if file_size > 5_000_000 {
                return delete(f).await;
            }

            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }

    Ok(HttpResponse::Ok().into())
}

fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    std::fs::create_dir_all("./tmp").unwrap();

    let ip = "0.0.0.0:3000";

    HttpServer::new(|| {
        App::new()
        .wrap(middleware::Logger::default())
        .app_data(web::PayloadConfig::default().limit(1))
        .service(
            web::resource("/")
                /*.app_data(actix_web::web::Bytes::configure(
                    |cfg| {
                        cfg.limit(1000000 * 250)
                    }
                ))*/
                .route(web::get().to(index))
                .route(web::post()
                .to(save_file)),
        )
    })
    .bind(ip)?
    .run()
    .await
}

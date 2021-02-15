pub mod util;

use util::result::Error;
use util::variables::FILE_SIZE_LIMIT;

#[macro_use]
extern crate lazy_static;
extern crate tree_magic;

use log::info;
use imagesize;
use ffprobe::ffprobe;
use tempfile::NamedTempFile;
use std::{fs::File, io::Write};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use actix_web::{App, HttpResponse, HttpServer, middleware, web};

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    if let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();

        /* let stream = stream::iter(vec![Ok(vec![1, 2, 3, 4, 5])]);
let mut reader = stream.into_async_read();
let mut buf = Vec::new();

assert!(reader.read_to_end(&mut buf).await.is_ok()); */
        // let mut reader = field.into_async_read();
        // let mut buf = Vec::new();

        // ? Read multipart data into a buffer.        
        let mut file_size: usize = 0;
        let mut buf: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_size += data.len();

            if file_size > *FILE_SIZE_LIMIT {
                return Err(Error::FileTooLarge { max_size: *FILE_SIZE_LIMIT });
            }

            buf.append(&mut data.to_vec());
        }

        // ? Find the content-type of the data.
        let content_type = tree_magic::from_u8(&buf);
        dbg!(&content_type);
        let s = &content_type[..];

        match s {
            /* jpg */ "image/jpeg" |
            /* png */ "image/png" |
            /* gif */ "image/gif"  => {
                if let Ok(size) = imagesize::blob_size(&buf) {
                    dbg!(size);
                } else {
                    return Err(Error::LabelMe)
                }
            }
            /*  mp4 */ "video/mp4" |
            /* webm */ "video/webm" => {
                let tmp = NamedTempFile::new().unwrap();
                let (mut tmp, path) = tmp.keep().unwrap();
                
                web::block(move || tmp.write_all(&buf).map(|_| tmp)).await
                    .map_err(|_| Error::LabelMe)?;
                
                let data = ffprobe(path).unwrap();
                let stream = data.streams.into_iter().next().unwrap();
                dbg!(stream.width, stream.height);
            }
            /* mp3 */ "audio/mpeg" => {

            }
            _ => {
                
            }
        }

        /*let fpath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));

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
            
            Err(Error::FileTooLarge)
        };

        let mut file_size: usize = 0;
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            file_size += data.len();

            if file_size > *FILE_SIZE_LIMIT {
                return delete(f).await;
            }

            f = web::block(move || f.write_all(&data).map(|_| f)).await
                .map_err(|_| Error::LabelMe)?;
        }

        drop(f);*/
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
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!("Starting Autumn server.");

    // std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
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

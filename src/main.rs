pub mod util;
pub mod db;

use util::result::Error;
use util::variables::{FILE_SIZE_LIMIT, HOST};

#[macro_use]
extern crate lazy_static;
extern crate tree_magic;

use log::info;
use imagesize;
use nanoid::nanoid;
use ffprobe::ffprobe;
use std::convert::TryFrom;
use tempfile::NamedTempFile;
use std::{fs::File, io::Write};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use actix_web::{App, HttpResponse, HttpServer, middleware, web};

enum Metadata {
    File,
    Image { width: usize, height: usize },
    Video { width: usize, height: usize },
    Audio
}

struct Attachment {
    id: String,
    filename: String,
    metadata: Metadata
}

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    if let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();

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

        let metadata = match s {
            /* jpg */ "image/jpeg" |
            /* png */ "image/png" |
            /* gif */ "image/gif"  => {
                if let Ok(imagesize::ImageSize { width, height }) = imagesize::blob_size(&buf) {
                    Metadata::Image { width, height }
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
                
                Metadata::Video {
                    width: TryFrom::try_from(stream.width.unwrap()).unwrap(),
                    height: TryFrom::try_from(stream.height.unwrap()).unwrap()
                }
            }
            /* mp3 */ "audio/mpeg" => {
                Metadata::Audio
            }
            _ => {
                Metadata::File
            }
        };

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

    db::connect().await;
    std::fs::create_dir_all("./files").unwrap();

    HttpServer::new(|| {
        App::new()
        .wrap(middleware::Logger::default())
        .service(
            web::resource("/")
                .route(web::get().to(index))
                .route(web::post()
                .to(save_file)),
        )
    })
    .bind(HOST.clone())?
    .run()
    .await
}

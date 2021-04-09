use crate::db::*;
use crate::config::get_tag;
use crate::util::result::Error;
use crate::util::variables::{USE_S3, LOCAL_STORAGE_PATH, get_s3_bucket};

use actix_web::{HttpRequest, HttpResponse, web::Query};
use image::{ImageError, io::Reader as ImageReader};
use tokio::io::AsyncReadExt;
use serde::Deserialize;
use std::path::PathBuf;
use mongodb::bson::doc;
use tokio::fs::File;
use std::io::Cursor;
use std::cmp;

#[derive(Deserialize, Debug)]
pub struct Resize {
    pub size: Option<isize>,
    pub width: Option<isize>,
    pub height: Option<isize>
}

pub fn try_resize(buf: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>, ImageError> {
    let mut bytes: Vec<u8> = Vec::new();

    ImageReader::new(Cursor::new(buf))
        .with_guessed_format()?
        .decode()?
        // resize_exact is about 2.5x slower,
        //  thumb approximation doesn't have terrible quality so it's fine to stick with
        //.resize_exact(width as u32, height as u32, image::imageops::FilterType::Gaussian)
        .thumbnail_exact(width as u32, height as u32)
        .write_to(&mut bytes, image::ImageOutputFormat::Png)?;

    Ok(bytes)
}

pub async fn fetch_file(id: &str, metadata: Metadata, resize: Option<Resize>) -> Result<(Vec<u8>, Option<String>), Error> {
    let mut contents = vec![];

    if *USE_S3 {
        let bucket = get_s3_bucket()?;
        let (data, code) = bucket.get_object(format!("/{}", id)).await.map_err(|_| Error::LabelMe)?;
        
        if code != 200 {
            return Err(Error::LabelMe);
        }

        contents = data;
    } else {
        let path: PathBuf = format!("{}/{}", *LOCAL_STORAGE_PATH, id)
            .parse()
            .map_err(|_| Error::IOError)?;

        let mut f = File::open(path.clone()).await.map_err(|_| Error::LabelMe)?;
        f.read_to_end(&mut contents).await.map_err(|_| Error::LabelMe)?;
    }

    if let Some(parameters) = resize {
        if let Metadata::Image { width, height } = metadata {
            let (target_width, target_height) =
                match (parameters.size, parameters.width, parameters.height) {
                    (Some(size), _, _) => (size, size),
                    (_, Some(w), Some(h)) =>
                        (cmp::min(width, w), cmp::min(height, h)),
                    (_, Some(w), _) => {
                        let w = cmp::min(width, w);
                        (w, (height as f32 * (w as f32 / width as f32)) as isize)
                    }
                    (_, _, Some(h)) => {
                        let h = cmp::min(height, h);
                        ((width as f32 * (h as f32 / height as f32)) as isize, h)
                    }
                    _ => return Ok((contents, None))
                };
            
            // There should be a way to do this zero-copy, but I can't be asked to figure it out right now.
            let cloned = contents.clone();
            if let Ok(bytes) = actix_web::web::block(move || try_resize(cloned, target_width as u32, target_height as u32)).await {
                return Ok((bytes, Some("image/png".to_string())))
            }
        }
    }

    Ok((contents, None))
}

pub async fn get(req: HttpRequest, resize: Query<Resize>) -> Result<HttpResponse, Error> {
    let tag = get_tag(&req)?;
    
    let id = req.match_info().query("filename");
    let file = find_file(id, &tag).await?;
    let (contents, content_type) = fetch_file(id, file.metadata, Some(resize.0)).await?;

    Ok(HttpResponse::Ok()
        .set_header("Content-Disposition", "inline")
        .content_type(content_type.unwrap_or(file.content_type))
        .body(contents))
}

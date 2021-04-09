use crate::db::*;
use crate::util::result::Error;
use crate::config::{ContentType, get_tag};
use crate::util::variables::{USE_S3, LOCAL_STORAGE_PATH, get_s3_bucket};

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use ffprobe::ffprobe;
use futures::{StreamExt, TryStreamExt};
use imagesize;
use mongodb::bson::to_document;
use nanoid::nanoid;
use serde_json::json;
use std::convert::TryFrom;
use std::io::Write;
use tempfile::NamedTempFile;
use content_inspector::inspect;

pub async fn option() -> HttpResponse {
    HttpResponse::Ok().into()
}

pub async fn post(req: HttpRequest, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let tag = get_tag(&req)?;

    if let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field
            .content_disposition()
            .ok_or_else(|| Error::FailedToReceive)?;
        let filename = content_type
            .get_filename()
            .ok_or_else(|| Error::FailedToReceive)?
            .to_string();

        // ? Read multipart data into a buffer.
        let mut file_size: usize = 0;
        let mut buf: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|_| Error::FailedToReceive)?;
            file_size += data.len();

            if file_size > tag.max_size {
                return Err(Error::FileTooLarge {
                    max_size: tag.max_size,
                });
            }

            buf.append(&mut data.to_vec());
        }

        // ? Find the content-type of the data.
        let content_type = tree_magic::from_u8(&buf);
        let s = &content_type[..];

        let metadata = match s {
            /* jpg */ "image/jpeg" |
            /* png */ "image/png" |
            /* gif */ "image/gif" |
            /* webp */ "image/webp"  => {
                if let Ok(imagesize::ImageSize { width, height }) = imagesize::blob_size(&buf) {
                    Metadata::Image {
                        width: TryFrom::try_from(width).unwrap(),
                        height: TryFrom::try_from(height).unwrap()
                    }
                } else {
                    Metadata::File
                }
            }
            /*  mp4 */ "video/mp4" |
            /* webm */ "video/webm" => {
                // ! FIXME: handle errors by falling back to File
                let tmp = NamedTempFile::new().map_err(|_| Error::IOError)?;
                let (mut tmp, path) = tmp.keep().map_err(|_| Error::IOError)?;
                buf = web::block(move || tmp.write_all(&buf).map(|_| buf)).await
                    .map_err(|_| Error::LabelMe)?;
                let data = ffprobe(path).map_err(|_| Error::ProbeError)?;
                let stream = data.streams.into_iter().next().ok_or_else(|| Error::ProbeError)?;
                Metadata::Video {
                    width: TryFrom::try_from(stream.width.ok_or(Error::ProbeError)?).unwrap(),
                    height: TryFrom::try_from(stream.height.ok_or(Error::ProbeError)?).unwrap()
                }
            }
            /* mp3 */ "audio/mpeg" => {
                Metadata::Audio
            }
            _ => {
                if inspect(&buf).is_text() {
                    Metadata::Text
                } else {
                    Metadata::File
                }
            }
        };

        if let Some(content_type) = &tag.restrict_content_type {
            if match (content_type, &metadata) {
                (ContentType::Image, Metadata::Image { .. }) => false,
                (ContentType::Video, Metadata::Video { .. }) => false,
                (ContentType::Audio, Metadata::Audio) => false,
                _ => true
            } {
                return Err(Error::LabelMe)
            }
        }

        let id = nanoid!(42);
        let file = crate::db::File {
            id,
            filename,
            metadata,
            content_type,
            size: file_size as isize
        };

        get_collection("attachments")
            .insert_one(to_document(&file).map_err(|_| Error::DatabaseError)?, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        if *USE_S3 {
            let bucket = get_s3_bucket()?;

            let (_, code) = bucket.put_object(format!("/{}", file.id), &buf)
                .await.unwrap();
            
            if code != 200 {
                return Err(Error::LabelMe)
            }
        } else {
            let path = format!("{}/{}", *LOCAL_STORAGE_PATH, &file.id);
            let mut f = web::block(|| std::fs::File::create(path))
                .await
                .map_err(|_| Error::IOError)?;

            web::block(move || f.write_all(&buf))
                .await
                .map_err(|_| Error::LabelMe)?;
        }

        Ok(HttpResponse::Ok().body(json!({ "id": file.id })))
    } else {
        Err(Error::MissingData)
    }
}

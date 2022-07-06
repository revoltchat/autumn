use crate::config::{get_tag, Config, ContentType};
use crate::db::*;
use crate::util::result::Error;
use crate::util::variables::{get_s3_bucket, LOCAL_STORAGE_PATH, USE_S3};

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use content_inspector::inspect;
use ffprobe::ffprobe;
use futures::{StreamExt, TryStreamExt};
use image::io::Reader as ImageReader;
use imagesize;
use nanoid::nanoid;
use serde_json::json;
use std::convert::TryInto;
use std::io::{Cursor, Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

pub fn determine_video_size(path: &std::path::Path) -> Result<(isize, isize), Error> {
    let data = ffprobe(path).map_err(|_| Error::ProbeError)?;

    // Take the first valid stream.
    for stream in data.streams {
        if let (Some(w), Some(h)) = (stream.width, stream.height) {
            if let (Ok(w), Ok(h)) = (w.try_into(), h.try_into()) {
                return Ok((w, h));
            }
        }
    }

    Err(Error::ProbeError)
}

pub async fn post(req: HttpRequest, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let config = Config::global();
    let (tag_id, tag) = get_tag(&req)?;

    if let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().ok_or(Error::FailedToReceive)?;
        let filename = content_type
            .get_filename()
            .ok_or(Error::FailedToReceive)?
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
                    if s == "image/jpeg" {
                        let mut bytes: Vec<u8> = Vec::new();
                        let mut cursor = Cursor::new(buf);

                        // Attempt to extract orientation data.
                        let exif_reader = exif::Reader::new();
                        let rotation = match exif_reader.read_from_container(&mut cursor) {
                            Ok(exif) => {
                                match exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                                    Some(orientation) => {
                                        match orientation.value.get_uint(0) {
                                            Some(v @ 1..=8) => v,
                                            _ => 0
                                        }
                                    }
                                    _ => 0
                                }
                            }
                            _ => 0
                        };

                        cursor.set_position(0);

                        // Re-encode JPEGs to remove EXIF data.
                        let image = ImageReader::new(cursor)
                            .with_guessed_format()
                            .map_err(|_| Error::IOError)?
                            .decode()
                            .map_err(|_| Error::IOError);

                            // See https://jdhao.github.io/2019/07/31/image_rotation_exif_info/
                            match &rotation {
                                2 => { image?.fliph() }
                                3 => { image?.rotate180() }
                                4 => { image?.rotate180().fliph() }
                                5 => { image?.rotate90().fliph() }
                                6 => { image?.rotate90() }
                                7 => { image?.rotate270().fliph() }
                                8 => { image?.rotate270() }
                                _ => { image? }
                            }
                            .write_to(&mut bytes, image::ImageOutputFormat::Jpeg(config.jpeg_quality))
                            .map_err(|_| Error::IOError)?;

                        buf = bytes;
                    }

                    Metadata::Image {
                        width: width.try_into().map_err(|_| Error::IOError)?,
                        height: height.try_into().map_err(|_| Error::IOError)?
                    }
                } else {
                    Metadata::File
                }
            }
            /*  mp4 */ "video/mp4" |
            /* webm */ "video/webm" |
            /*  mov */ "video/quicktime" => {
                let ext = match s {
                    "video/mp4" => "mp4",
                    "video/webm" => "webm",
                    "video/quicktime" => "mov",
                    _ => unreachable!()
                };

                let mut tmp = NamedTempFile::new().map_err(|_| Error::IOError)?;
                tmp.write_all(&buf).map_err(|_| Error::IOError)?;

                if let Ok(Ok(((width, height), tmp))) = web::block(move || determine_video_size(tmp.path()).map(|t| (t, tmp))).await {
                    buf = vec![];
                    let out_tmp = NamedTempFile::new().map_err(|_| Error::IOError)?;
                    let out_tmp = web::block(move ||
                        Command::new("ffmpeg")
                            .args(&[
                                "-y",                                               // Overwrite the temporary file.
                                "-i", tmp.path().to_str().ok_or(Error::IOError)?,   // Read the original uploaded file.
                                "-map_metadata", "-1",                              // Strip any metadata.
                                "-c:v", "copy", "-c:a", "copy",                     // Copy video / audio data to new file.
                                "-f", ext,                                          // Select the correct file format.
                                out_tmp.path().to_str().ok_or(Error::IOError)?])    // Save to new temporary file.
                            .output()
                            .map(|_| out_tmp)
                            .map_err(|_| Error::IOError)
                    )
                    .await
                    .map_err(|_| Error::BlockingError)?
                    .map_err(|_| Error::IOError)?;

                    let mut file = web::block(move || std::fs::File::open(out_tmp.path()).map(|f| (f, out_tmp)))
                        .await
                        .map_err(|_| Error::BlockingError)?
                        .map_err(|_| Error::IOError)?;

                    buf = web::block(move || file.0.read_to_end(&mut buf).map(|_| buf))
                        .await
                        .map_err(|_| Error::BlockingError)?
                        .map_err(|_| Error::IOError)?;

                    Metadata::Video {
                        width,
                        height
                    }
                } else {
                    Metadata::File
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
            if !matches!(
                (content_type, &metadata),
                (ContentType::Image, Metadata::Image { .. })
                    | (ContentType::Video, Metadata::Video { .. })
                    | (ContentType::Audio, Metadata::Audio)
            ) {
                return Err(Error::FileTypeNotAllowed);
            }
        }

        let id = if tag.use_ulid { ulid::Ulid::new().to_string() } else { nanoid!(42) };
        let file = crate::db::File {
            id,
            tag: tag_id.clone(),
            filename,
            metadata,
            content_type,
            size: buf.len() as isize,
            deleted: None,
            reported: None,
        };

        get_collection("attachments")
            .insert_one(&file, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        if *USE_S3 {
            let bucket = get_s3_bucket(&tag_id)?;

            let (_, code) = bucket
                .put_object(format!("/{}", file.id), &buf)
                .await
                .map_err(|_| Error::S3Error)?;

            if code != 200 {
                return Err(Error::S3Error);
            }
        } else {
            let path = format!("{}/{}", *LOCAL_STORAGE_PATH, &file.id);
            let mut f = web::block(|| std::fs::File::create(path))
                .await
                .map_err(|_| Error::BlockingError)?
                .map_err(|_| Error::IOError)?;

            web::block(move || f.write_all(&buf))
                .await
                .map_err(|_| Error::BlockingError)?
                .map_err(|_| Error::IOError)?;
        }

        Ok(HttpResponse::Ok().json(json!({ "id": file.id })))
    } else {
        Err(Error::MissingData)
    }
}

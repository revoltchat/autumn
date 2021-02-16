use crate::db::*;
use crate::util::result::Error;

use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::HttpRequest;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use std::path::PathBuf;

pub async fn static_serve(req: HttpRequest) -> Result<NamedFile, Error> {
    let id = req.match_info().query("filename");

    let content_type = get_collection("attachments")
        .find_one(
            doc! {
                "_id": id,
                "message_id": {
                    "$exists": true
                }
            },
            FindOneOptions::builder()
                .projection(doc! { "_id": 0, "content_type": 1 })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError)?
        .ok_or_else(|| Error::NotFound)?
        .get_str("content_type")
        .map_err(|_| Error::DatabaseError)?
        .parse::<mime::Mime>()
        .map_err(|_| Error::LabelMe)?;

    let path: PathBuf = format!("./files/{}", id)
        .parse()
        .map_err(|_| Error::IOError)?;

    Ok(NamedFile::open(path)
        .map_err(|_| Error::IOError)?
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![],
        })
        .set_content_type(content_type))
}

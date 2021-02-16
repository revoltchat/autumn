use crate::db::*;
use crate::util::result::Error;

use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::HttpRequest;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use std::path::PathBuf;

pub async fn get(req: HttpRequest) -> Result<NamedFile, Error> {
    let id = req.match_info().query("filename");

    let doc = get_collection("attachments")
        .find_one(
            doc! {
                "_id": id,
                //"message_id": {
                    //"$exists": true
                //}
            },
            FindOneOptions::builder()
                .projection(doc! { "_id": 0, "filename": 1, "content_type": 1 })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError)?
        .ok_or_else(|| Error::NotFound)?;

    let filename = doc
        .get_str("filename")
        .map_err(|_| Error::DatabaseError)?
        .to_string();

    let content_type = doc
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
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(filename)],
        })
        .set_content_type(content_type))
}

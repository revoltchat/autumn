use crate::db::*;
use crate::util::result::Error;
use crate::util::variables::{USE_S3, get_s3_bucket};

use actix_web::{HttpRequest, HttpResponse};
use mongodb::bson::{doc, from_document};
use mongodb::options::FindOneOptions;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
pub struct PartialFile {
    pub filename: String,
    pub content_type: String,
}

pub async fn find_file(id: &str) -> Result<PartialFile, Error> {
    let doc = get_collection("attachments")
        .find_one(
            doc! {
                "_id": id,
                /*"message_id": {
                    "$exists": true
                }*/
            },
            FindOneOptions::builder()
                .projection(doc! { "_id": 0, "filename": 1, "content_type": 1 })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError)?
        .ok_or_else(|| Error::NotFound)?;
    
    from_document(doc).map_err(|_| Error::DatabaseError)
}

pub async fn fetch_file(id: &str) -> Result<Vec<u8>, Error> {
    let mut contents = vec![];

    if *USE_S3 {
        let bucket = get_s3_bucket()?;
        let (data, code) = bucket.get_object(format!("/{}", id)).await.map_err(|_| Error::LabelMe)?;
        
        if code != 200 {
            return Err(Error::LabelMe);
        }

        contents = data;
    } else {
        let path: PathBuf = format!("./files/{}", id)
            .parse()
            .map_err(|_| Error::IOError)?;

        let mut f = File::open(path.clone()).await.map_err(|_| Error::LabelMe)?;
        f.read_to_end(&mut contents).await.map_err(|_| Error::LabelMe)?;
    }

    Ok(contents)
}

pub async fn get(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id = req.match_info().query("filename");
    let file = find_file(id).await?;
    let contents = fetch_file(id).await?;

    Ok(HttpResponse::Ok()
        .set_header("Content-Disposition", "inline")
        .content_type(file.content_type)
        .body(contents))
}

use crate::config::Tag;
use crate::util::result::Error;
use crate::util::variables::{
    get_s3_bucket, LOCAL_STORAGE_PATH, MONGO_DATABASE, MONGO_URI, USE_S3,
};

use actix_web::web;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static DBCONN: OnceCell<Client> = OnceCell::new();

pub async fn connect() {
    let client = Client::with_uri_str(&*MONGO_URI)
        .await
        .expect("Failed to init db connection.");

    DBCONN.set(client).unwrap();
}

pub fn get_collection(collection: &str) -> Collection<File> {
    DBCONN
        .get()
        .unwrap()
        .database(&MONGO_DATABASE)
        .collection(collection)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Metadata {
    File,
    Text,
    Image { width: isize, height: isize },
    Video { width: isize, height: isize },
    Audio,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: String,
    pub tag: String,
    pub filename: String,
    pub metadata: Metadata,
    pub content_type: String,
    pub size: isize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reported: Option<bool>,
}

impl File {
    pub async fn delete_in_storage(&self) -> Result<(), Error> {
        if *USE_S3 {
            let bucket = get_s3_bucket(&self.tag)?;

            let (_, code) = bucket
                .delete_object(format!("/{}", &self.id))
                .await
                .map_err(|_| Error::S3Error)?;

            if code != 200 {
                return Err(Error::S3Error);
            }
        } else {
            let path = format!("{}/{}", *LOCAL_STORAGE_PATH, &self.id);
            web::block(|| std::fs::remove_file(path))
                .await
                .map_err(|_| Error::BlockingError)?
                .map_err(|_| Error::IOError)?;
        }

        Ok(())
    }

    pub async fn delete(self) -> Result<(), Error> {
        self.delete_in_storage().await.ok();

        get_collection("attachments")
            .delete_one(doc! { "_id": &self.id }, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        println!("Deleted attachment {}", self.id);
        Ok(())
    }
}

pub async fn find_file(id: &str, tag: (String, &Tag)) -> Result<File, Error> {
    let mut query = doc! { "_id": id, "tag": tag.0 };

    if !&tag.1.serve_if_field_present.is_empty() {
        let mut or = vec![];
        for field in &tag.1.serve_if_field_present {
            or.push(doc! {
                field: {
                    "$exists": true
                }
            });
        }

        query.insert("$or", or);
    }

    get_collection("attachments")
        .find_one(query, None)
        .await
        .map_err(|_| Error::DatabaseError)?
        .ok_or(Error::NotFound)
}

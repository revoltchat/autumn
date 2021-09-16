use crate::config::Tag;
use crate::util::result::Error;
use crate::util::variables::MONGO_URI;

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
        .database("revolt")
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

pub async fn find_file(id: &str, tag: (String, &Tag)) -> Result<File, Error> {
    let mut query = doc! { "_id": id, "tag": tag.0 };

    if let Some(field) = &tag.1.serve_if_field_present {
        query.insert(field, doc! { "$exists": true });
    }

    get_collection("attachments")
        .find_one(query, None)
        .await
        .map_err(|_| Error::DatabaseError)?
        .ok_or_else(|| Error::NotFound)
}

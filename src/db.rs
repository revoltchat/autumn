use crate::util::variables::MONGO_URI;

use serde::{Serialize, Deserialize};
use mongodb::{Client, Collection};
use once_cell::sync::OnceCell;

static DBCONN: OnceCell<Client> = OnceCell::new();

pub async fn connect() {
    let client = Client::with_uri_str(&MONGO_URI)
        .await
        .expect("Failed to init db connection.");

    DBCONN.set(client).unwrap();
}

pub fn get_collection(collection: &str) -> Collection {
    DBCONN.get().unwrap().database("revolt").collection(collection)
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Metadata {
    File,
    Image { width: isize, height: isize },
    Video { width: isize, height: isize },
    Audio
}

#[derive(Serialize, Deserialize)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: String,
    pub filename: String,
    pub metadata: Metadata
}

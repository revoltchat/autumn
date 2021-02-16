use crate::util::variables::MONGO_URI;

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

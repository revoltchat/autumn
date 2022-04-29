use crate::util::result::Error;

use s3::{creds::Credentials, Region};
use std::env;

lazy_static! {
    // Application Settings
    pub static ref CONFIG: String =
        env::var("AUTUMN_CONFIG").unwrap_or_else(|_| String::from("Autumn.toml"));
    pub static ref HOST: String =
        env::var("AUTUMN_HOST").expect("Missing AUTUMN_HOST environment variable.");
    pub static ref MONGO_URI: String =
        env::var("AUTUMN_MONGO_URI").expect("Missing AUTUMN_MONGO_URI environment variable.");
    pub static ref MONGO_DATABASE: String =
        env::var("AUTUMN_MONGO_DATABASE").unwrap_or_else(|_| "revolt".to_string());
    pub static ref CORS_ALLOWED_ORIGIN: String =
        env::var("AUTUMN_CORS_ALLOWED_ORIGIN").expect("Missing AUTUMN_CORS_ALLOWED_ORIGIN environment variable.");

    // Storage Settings
    pub static ref LOCAL_STORAGE_PATH: String =
        env::var("AUTUMN_LOCAL_STORAGE_PATH").unwrap_or_else(|_| "./files".to_string());
    pub static ref S3_REGION: Region = Region::Custom {
        region: env::var("AUTUMN_S3_REGION").unwrap_or_else(|_| "".to_string()),
        endpoint: env::var("AUTUMN_S3_ENDPOINT").unwrap_or_else(|_| "".to_string())
    };
    pub static ref S3_CREDENTIALS: Credentials = Credentials::default().unwrap();

    // Application Flags
    pub static ref USE_S3: bool = env::var("AUTUMN_S3_REGION").is_ok() && env::var("AUTUMN_S3_ENDPOINT").is_ok();
}

pub fn get_s3_bucket(bucket: &str) -> Result<s3::Bucket, Error> {
    s3::Bucket::new_with_path_style(bucket, S3_REGION.clone(), S3_CREDENTIALS.clone())
        .map_err(|_| Error::S3Error)
}

use std::env;

lazy_static! {
    // Application Settings
    pub static ref HOST: String =
        env::var("AUTUMN_HOST").expect("Missing AUTUMN_HOST environment variable.");
    pub static ref MONGO_URI: String =
        env::var("AUTUMN_MONGO_URI").expect("Missing AUTUMN_MONGO_URI environment variable.");
    pub static ref FILE_SIZE_LIMIT: usize =
        env::var("AUTUMN_FILE_SIZE_LIMIT").expect("Missing AUTUMN_FILE_SIZE_LIMIT environment variable.").parse().unwrap();
}

use std::env;

lazy_static! {
    // Application Settings
    pub static ref FILE_SIZE_LIMIT: usize =
        env::var("AUTUMN_FILE_SIZE_LIMIT").expect("Missing AUTUMN_FILE_SIZE_LIMIT environment variable.").parse().unwrap();
}

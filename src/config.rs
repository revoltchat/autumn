use actix_web::HttpRequest;
use serde::{Serialize, Deserialize};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::util::result::Error;

#[derive(Serialize, Deserialize, Debug)]
pub enum ContentType {
    Image,
    Video,
    Audio
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub max_size: usize,
    pub enabled: Option<bool>,
    pub serve_if_field_present: Option<String>,
    pub restrict_content_type: Option<ContentType>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    tags: HashMap<String, Tag>
}

static INSTANCE: OnceCell<Config> = OnceCell::new();

impl Config {
    pub fn global() -> &'static Config {
        INSTANCE.get().expect("Config is not initialized.")
    }

    pub fn init() -> std::io::Result<()> {
        let mut file = File::open("Autumn.toml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = toml::from_str(&contents).unwrap();
        INSTANCE.set(config).expect("Failed to set global config.");
        Ok(())
    }
}

pub fn get_tag(request: &HttpRequest) -> Result<&Tag, Error> {
    let id = request.match_info().query("tag");
    let config = Config::global();

    if let Some(tag) = config.tags.get(id) {
        if let Some(false) = tag.enabled {
            return Err(Error::LabelMe)
        }

        Ok(tag)
    } else {
        Err(Error::LabelMe)
    }
}

use crate::config::Config;

use actix_web::HttpResponse;
use serde_json::json;

pub async fn get() -> HttpResponse {
    let config = Config::global();
    let body = json!({
        "autumn": "1.0.0",
        "tags": config.tags
    });

    HttpResponse::Ok().body(body)
}

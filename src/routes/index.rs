use crate::config::Config;

use serde_json::json;
use actix_web::HttpResponse;

pub async fn get() -> HttpResponse {
    let config = Config::global();
    let body = json!({
        "autumn": "1.0.0",
        "tags": config.tags
    });

    HttpResponse::Ok()
        .body(body)
}

use serde_json;
use serde::Serialize;
use std::fmt::Display;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Error {
    FileTooLarge {
        max_size: usize
    },
    LabelMe,
}

impl Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match &self {
            Error::FileTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            Error::LabelMe => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::build(self.status_code())
            .content_type("application/json")
            .body(body)
    }
}

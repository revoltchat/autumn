use serde_json;
use serde::Serialize;
use std::fmt::Display;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError, dev::HttpResponseBuilder};

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

        /*let mut resp = HttpResponse::new(self.status_code());
        let mut buf = actix_web::web::BytesMut::new();
        let _ = write!(Writer(&mut buf), "{}", self);
        resp.headers_mut().insert(
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        resp.set_body(actix_web::dev::Body::from(buf))*/

        HttpResponse::build(self.status_code())
            .content_type("application/json")
            .body(body)
    }
}

/*impl Responder for Error {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        ready(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    }
}*/

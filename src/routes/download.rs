use crate::util::result::Error;
use super::serve::{find_file, fetch_file};

use actix_web::{HttpRequest, HttpResponse};

pub async fn get(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id = req.match_info().query("filename");
    let file = find_file(id).await?;
    let contents = fetch_file(id).await?;

    Ok(HttpResponse::Ok()
        .set_header("Content-Disposition", format!("attachment; filename=\"{}\"", file.filename))
        .content_type(file.content_type)
        .body(contents))
}

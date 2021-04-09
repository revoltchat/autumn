use crate::config::get_tag;
use crate::db::find_file;
use crate::util::result::Error;

use super::serve::fetch_file;

use actix_web::{HttpRequest, HttpResponse};

pub async fn get(req: HttpRequest) -> Result<HttpResponse, Error> {
    let tag = get_tag(&req)?;

    let id = req.match_info().query("filename");
    let file = find_file(id, tag).await?;
    let (contents, _) = fetch_file(id, file.metadata, None).await?;

    Ok(HttpResponse::Ok()
        .set_header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file.filename),
        )
        .content_type(file.content_type)
        .body(contents))
}

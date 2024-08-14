use crate::config::{get_tag, Config};
use crate::db::find_file;
use crate::util::result::Error;

use super::serve::fetch_file;

use actix_web::{HttpRequest, HttpResponse};

pub async fn get(req: HttpRequest) -> Result<HttpResponse, Error> {
    let tag = get_tag(&req)?;

    let id = req.match_info().query("filename");
    let file = find_file(id, tag.clone()).await?;

    if let Some(true) = file.deleted {
        return Err(Error::NotFound);
    }

    let config = Config::global();
    if config.filter.content_types.contains(&file.content_type) {
        return Err(Error::ContentTypeNotAllowed);
    }

    let (contents, _) = fetch_file(id, &tag.0, file.metadata, None).await?;

    Ok(HttpResponse::Ok()
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file.filename),
        ))
        .insert_header(("Cache-Control", crate::CACHE_CONTROL))
        .content_type(file.content_type)
        .body(contents))
}

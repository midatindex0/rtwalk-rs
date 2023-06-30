use actix_files::NamedFile;
use actix_web::{get, web, Result};

#[get("/pfp/{username}")]
async fn pfp(path: web::Path<(String,)>) -> Result<NamedFile> {
    let (mut username,) = path.into_inner();

    if !username.ends_with(".png") {
        username.push_str(".png");
    }

    // TODO: Sanitize
    Ok(NamedFile::open(format!("{}/{}", "data/pfp", username))?)
}

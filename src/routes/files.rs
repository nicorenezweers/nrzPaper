//! Dateien ausliefern (mit Range-Anfragen für die PDF-Vorschau) – für die API und für
//! öffentliche Freigabelinks: `/share/<token>/file` braucht keine Anmeldung.

use crate::error::AppResult;
use crate::services::{documents, share_links};
use crate::storage::{FileResponse, RangeHeader};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::{get, routes, Route, State};
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![shared_file]
}

#[derive(rocket::Responder)]
pub enum FileReply {
    File(FileResponse),
    Unsatisfiable(Custom<String>),
}

/// Datei ausliefern: `?download` als Anhang, sonst inline; `Range` → 206.
pub async fn serve(
    pool: &SqlitePool,
    org: &str,
    id: &str,
    range: Option<&str>,
    download: bool,
) -> AppResult<FileReply> {
    let doc = documents::get(pool, org, id).await?;
    Ok(
        match FileResponse::load(pool, &doc, range, download).await? {
            Ok(f) => FileReply::File(f),
            Err(size) => FileReply::Unsatisfiable(Custom(
                Status::RangeNotSatisfiable,
                format!("bytes */{size}"),
            )),
        },
    )
}

#[get("/share/<token>/file?<download>")]
async fn shared_file(
    token: &str,
    download: Option<bool>,
    range: RangeHeader,
    pool: &State<SqlitePool>,
) -> AppResult<FileReply> {
    let (_, doc) = share_links::resolve(pool, token, false).await?;
    serve(
        pool,
        &doc.organization_id,
        &doc.id,
        range.0.as_deref(),
        download.unwrap_or(false),
    )
    .await
}

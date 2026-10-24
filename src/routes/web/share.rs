//! Öffentliche Freigabe: `/share/<token>` zeigt das Dokument ohne Anmeldung,
//! `/share/<token>/file` liefert die Datei (mit Range für die PDF-Vorschau).

use super::{serve_file, FileReply, Page};
use crate::error::AppResult;
use crate::services::share_links;
use crate::storage::RangeHeader;
use crate::templates::{render, SharePage};
use rocket::{get, routes, Route, State};
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![page, file]
}

#[get("/share/<token>?<find>")]
async fn page(token: &str, find: Option<&str>, pool: &State<SqlitePool>) -> AppResult<Page> {
    let (link, doc) = share_links::resolve(pool, token, true).await?;
    Ok(Page::Html(render(&SharePage {
        doc,
        token: token.to_string(),
        link,
        find: find.unwrap_or("").to_string(),
    })?))
}

#[get("/share/<token>/file?<download>")]
async fn file(
    token: &str,
    download: Option<bool>,
    range: RangeHeader,
    pool: &State<SqlitePool>,
) -> AppResult<FileReply> {
    let (_, doc) = share_links::resolve(pool, token, false).await?;
    serve_file(
        pool,
        &doc.organization_id,
        &doc.id,
        range.0.as_deref(),
        download.unwrap_or(false),
    )
    .await
}

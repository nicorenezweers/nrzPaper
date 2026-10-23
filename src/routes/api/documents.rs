//! Dokumente über die API (`/api/v1/organizations/<org>/documents…`), angelehnt an Papra.

use super::member;
use crate::auth::guards::ApiUser;
use crate::db::models::{DocumentSource, Permission};
use crate::error::{AppError, AppResult};
use crate::ocr::OcrQueue;
use crate::routes::upload::UploadForm;
use crate::routes::web::{serve_file, FileReply};
use crate::services::documents::{self, Ingested, NewDocument};
use crate::storage::RangeHeader;
use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![upload, list, list_trash, get_one, file, update, trash, restore, reprocess]
}

/// Upload (multipart, Feld `file`). Mehrere Dateien → mehrere Dokumente.
#[post("/organizations/<org>/documents", data = "<form>")]
async fn upload(
    api: ApiUser,
    org: &str,
    form: Form<UploadForm>,
    pool: &State<SqlitePool>,
    queue: &State<OcrQueue>,
) -> AppResult<(Status, Value)> {
    api.require(Permission::DocumentsCreate)?;
    member(pool, &api, org).await?;
    let mut docs = Vec::new();
    for f in form.into_inner().file {
        let new = NewDocument {
            organization_id: org,
            user_id: Some(&api.user.id),
            source: DocumentSource::Api,
            file_name: &f.name,
            data: f.data,
        };
        let outcome = documents::ingest(pool, queue, new).await?;
        docs.push(json!({ "document": outcome.document(), "restored": matches!(outcome, Ingested::Restored(_)) }));
    }
    if docs.is_empty() {
        return Err(AppError::bad("Keine Datei im Feld „file“."));
    }
    let body = if docs.len() == 1 {
        docs.remove(0)
    } else {
        json!({ "documents": docs })
    };
    Ok((Status::Created, body))
}

#[get("/organizations/<org>/documents?<pageIndex>&<pageSize>")]
#[allow(non_snake_case)]
async fn list(
    api: ApiUser,
    org: &str,
    pageIndex: Option<u32>,
    pageSize: Option<u32>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    api.require(Permission::DocumentsRead)?;
    member(pool, &api, org).await?;
    let (docs, total) = documents::list(
        pool,
        org,
        false,
        pageIndex.unwrap_or(0),
        pageSize.unwrap_or(100),
    )
    .await?;
    Ok(json!({ "documents": docs, "documentsCount": total }))
}

#[get("/organizations/<org>/documents/deleted?<pageIndex>&<pageSize>")]
#[allow(non_snake_case)]
async fn list_trash(
    api: ApiUser,
    org: &str,
    pageIndex: Option<u32>,
    pageSize: Option<u32>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    api.require(Permission::DocumentsRead)?;
    member(pool, &api, org).await?;
    let (docs, total) = documents::list(
        pool,
        org,
        true,
        pageIndex.unwrap_or(0),
        pageSize.unwrap_or(100),
    )
    .await?;
    Ok(json!({ "documents": docs, "documentsCount": total }))
}

#[get("/organizations/<org>/documents/<id>", rank = 2)]
async fn get_one(api: ApiUser, org: &str, id: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    api.require(Permission::DocumentsRead)?;
    member(pool, &api, org).await?;
    Ok(json!({ "document": documents::get(pool, org, id).await? }))
}

#[get("/organizations/<org>/documents/<id>/file")]
async fn file(
    api: ApiUser,
    org: &str,
    id: &str,
    range: RangeHeader,
    pool: &State<SqlitePool>,
) -> AppResult<FileReply> {
    api.require(Permission::DocumentsRead)?;
    member(pool, &api, org).await?;
    serve_file(pool, org, id, range.0.as_deref(), false).await
}

#[derive(Deserialize)]
struct DocumentUpdate {
    name: Option<String>,
    content: Option<String>,
}

#[patch("/organizations/<org>/documents/<id>", data = "<body>")]
async fn update(
    api: ApiUser,
    org: &str,
    id: &str,
    body: Json<DocumentUpdate>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    api.require(Permission::DocumentsUpdate)?;
    member(pool, &api, org).await?;
    let user = Some(api.user.id.as_str());
    if let Some(name) = &body.name {
        documents::rename(pool, org, id, name, user).await?;
    }
    if let Some(content) = &body.content {
        documents::set_content(pool, org, id, content, user).await?;
    }
    Ok(json!({ "document": documents::get(pool, org, id).await? }))
}

/// In den Papierkorb (wie Papra: DELETE löscht nicht sofort endgültig).
#[delete("/organizations/<org>/documents/<id>")]
async fn trash(api: ApiUser, org: &str, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    api.require(Permission::DocumentsDelete)?;
    member(pool, &api, org).await?;
    documents::trash(pool, org, id, Some(&api.user.id)).await?;
    Ok(Status::NoContent)
}

#[post("/organizations/<org>/documents/<id>/restore")]
async fn restore(api: ApiUser, org: &str, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    api.require(Permission::DocumentsUpdate)?;
    member(pool, &api, org).await?;
    documents::restore(pool, org, id, Some(&api.user.id)).await?;
    Ok(Status::NoContent)
}

#[post("/organizations/<org>/documents/<id>/reprocess")]
async fn reprocess(
    api: ApiUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
    queue: &State<OcrQueue>,
) -> AppResult<Status> {
    api.require(Permission::DocumentsUpdate)?;
    member(pool, &api, org).await?;
    documents::reprocess(pool, queue, org, id).await?;
    Ok(Status::Accepted)
}

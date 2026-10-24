//! Dokumente über die API (`/api/v1/organizations/<org>/documents…`), angelehnt an Papra.

use super::member;
use crate::auth::guards::ApiUser;
use crate::db::models::Document;
use crate::db::models::{DocumentSource, Permission};
use crate::error::{AppError, AppResult};
use crate::ocr::OcrQueue;
use crate::routes::upload::UploadForm;
use crate::routes::web::{serve_file, FileReply};
use crate::search;
use crate::services::documents::{self, Ingested, NewDocument};
use crate::services::{share_links, tags};
use crate::storage::RangeHeader;
use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![
        upload,
        list,
        list_trash,
        statistics,
        get_one,
        file,
        update,
        trash,
        restore,
        reprocess,
        activity,
        list_shares,
        create_share,
        revoke_share
    ]
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

/// Dokumente als JSON, jeweils mit ihren Tags.
async fn with_tags(pool: &SqlitePool, docs: Vec<Document>) -> AppResult<Vec<Value>> {
    let ids: Vec<String> = docs.iter().map(|d| d.id.clone()).collect();
    let mut map = tags::for_documents(pool, &ids).await?;
    Ok(docs
        .into_iter()
        .map(|d| {
            let tags = map.remove(&d.id).unwrap_or_default();
            let mut v = json!(d);
            v["tags"] = json!(tags);
            v
        })
        .collect())
}

/// Liste, optional mit Suche (`searchQuery`, gleiche Syntax wie in der Oberfläche) und auf
/// einen Ordner beschränkt (`folderId`: Dokumente mit einem Tag des Ordners oder seiner Unterordner).
#[allow(clippy::too_many_arguments)]
#[get("/organizations/<org>/documents?<pageIndex>&<pageSize>&<searchQuery>&<folderId>")]
#[allow(non_snake_case)]
async fn list(
    api: ApiUser,
    org: &str,
    pageIndex: Option<u32>,
    pageSize: Option<u32>,
    searchQuery: Option<&str>,
    folderId: Option<&str>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    api.require(Permission::DocumentsRead)?;
    member(pool, &api, org).await?;
    let (page, size) = (pageIndex.unwrap_or(0), pageSize.unwrap_or(100));
    let mut query = search::parse(searchQuery.unwrap_or("").trim());
    if let Some(folder) = folderId.filter(|f| !f.is_empty()) {
        let tree = crate::services::folders::tree(pool, org).await?;
        if !tree.iter().any(|f| f.id == folder) {
            return Err(AppError::NotFound);
        }
        query.any_tag_ids = Some(crate::services::folders::subtree_tags(&tree, folder));
    }
    let (docs, total) = search::search(pool, org, &query, page, size).await?;
    Ok(json!({ "documents": with_tags(pool, docs).await?, "documentsCount": total }))
}

/// Zahlen für Übersichten.
#[get("/organizations/<org>/documents/statistics")]
async fn statistics(api: ApiUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    api.require(Permission::DocumentsRead)?;
    member(pool, &api, org).await?;
    let (count, size, deleted, pending): (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE deleted_at IS NULL),
                coalesce(sum(size) FILTER (WHERE deleted_at IS NULL), 0),
                count(*) FILTER (WHERE deleted_at IS NOT NULL),
                count(*) FILTER (WHERE deleted_at IS NULL AND ocr_status IN ('pending', 'processing'))
         FROM documents WHERE organization_id = ?",
    )
    .bind(org)
    .fetch_one(pool.inner())
    .await?;
    Ok(json!({ "organizationStats": {
        "documentsCount": count, "documentsSize": size, "deletedDocumentsCount": deleted, "ocrPendingCount": pending
    } }))
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
    let doc = documents::get(pool, org, id).await?;
    Ok(json!({ "document": with_tags(pool, vec![doc]).await?.remove(0) }))
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

/// Verlauf eines Dokuments, neueste zuerst.
#[get("/organizations/<org>/documents/<id>/activity?<pageIndex>&<pageSize>")]
#[allow(non_snake_case)]
async fn activity(
    api: ApiUser,
    org: &str,
    id: &str,
    pageIndex: Option<u32>,
    pageSize: Option<u32>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    api.require(Permission::DocumentsRead)?;
    member(pool, &api, org).await?;
    documents::get(pool, org, id).await?;
    let size = pageSize.unwrap_or(50).clamp(1, 200) as i64;
    let rows: Vec<(String, String, Option<String>, Option<String>, String, i64)> = sqlx::query_as(
        "SELECT a.id, a.event, a.user_id, u.username, a.data, a.created_at
         FROM document_activity a LEFT JOIN users u ON u.id = a.user_id
         WHERE a.document_id = ? ORDER BY a.created_at DESC, a.id DESC LIMIT ? OFFSET ?",
    )
    .bind(id)
    .bind(size)
    .bind(pageIndex.unwrap_or(0) as i64 * size)
    .fetch_all(pool.inner())
    .await?;
    let activities: Vec<Value> = rows
        .into_iter()
        .map(|(id, event, user_id, username, data, at)| {
            json!({
                "id": id, "event": event, "userId": user_id, "username": username,
                "data": serde_json::from_str::<Value>(&data).unwrap_or_default(), "createdAt": at,
            })
        })
        .collect();
    Ok(json!({ "activities": activities }))
}

#[get("/organizations/<org>/documents/<id>/share-links")]
async fn list_shares(
    api: ApiUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    api.require(Permission::DocumentsRead)?;
    member(pool, &api, org).await?;
    Ok(json!({ "shareLinks": share_links::list(pool, org, id).await? }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewShare {
    expires_in_days: Option<u32>,
}

/// Freigabelink anlegen; `url` und `token` gibt es nur in dieser Antwort.
#[post("/organizations/<org>/documents/<id>/share-links", data = "<body>")]
async fn create_share(
    api: ApiUser,
    org: &str,
    id: &str,
    body: Option<Json<NewShare>>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    api.require(Permission::DocumentsUpdate)?;
    member(pool, &api, org).await?;
    let days = body.and_then(|b| b.expires_in_days);
    let (link, token) = share_links::create(pool, org, id, Some(&api.user.id), days).await?;
    Ok((
        Status::Created,
        json!({ "shareLink": link, "token": token, "path": format!("/share/{token}") }),
    ))
}

#[delete("/organizations/<org>/documents/<id>/share-links/<link>")]
async fn revoke_share(
    api: ApiUser,
    org: &str,
    id: &str,
    link: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    api.require(Permission::DocumentsUpdate)?;
    member(pool, &api, org).await?;
    share_links::revoke(pool, org, id, link, Some(&api.user.id)).await?;
    Ok(Status::NoContent)
}

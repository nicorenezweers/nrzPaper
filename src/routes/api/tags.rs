//! Tags über die API (`/api/v1/organizations/<org>/tags…`) und Tags an Dokumenten.

use super::member;
use crate::auth::guards::ApiUser;
use crate::db::models::Permission;
use crate::error::AppResult;
use crate::services::tags;
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![
        list,
        create,
        update,
        remove,
        add_to_document,
        remove_from_document
    ]
}

#[get("/organizations/<org>/tags")]
async fn list(api: ApiUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    api.require(Permission::TagsRead)?;
    member(pool, &api, org).await?;
    let tags: Vec<Value> = tags::list(pool, org)
        .await?
        .into_iter()
        .map(|(t, n)| {
            let mut v = json!(t);
            v["documentsCount"] = json!(n);
            v
        })
        .collect();
    Ok(json!({ "tags": tags }))
}

#[derive(Deserialize)]
struct TagBody {
    name: String,
    color: Option<String>,
    description: Option<String>,
}

#[post("/organizations/<org>/tags", data = "<body>")]
async fn create(
    api: ApiUser,
    org: &str,
    body: Json<TagBody>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    api.require(Permission::TagsCreate)?;
    member(pool, &api, org).await?;
    let color = body
        .color
        .clone()
        .unwrap_or_else(|| tags::PALETTE[0].to_string());
    let tag = tags::create(
        pool,
        org,
        &body.name,
        &color,
        body.description.as_deref().unwrap_or(""),
    )
    .await?;
    Ok((Status::Created, json!({ "tag": tag })))
}

#[put("/organizations/<org>/tags/<id>", data = "<body>")]
async fn update(
    api: ApiUser,
    org: &str,
    id: &str,
    body: Json<TagBody>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    api.require(Permission::TagsUpdate)?;
    member(pool, &api, org).await?;
    let old = tags::get(pool, org, id).await?;
    let color = body.color.clone().unwrap_or(old.color);
    let description = body.description.clone().unwrap_or(old.description);
    let tag = tags::update(pool, org, id, &body.name, &color, &description).await?;
    Ok(json!({ "tag": tag }))
}

#[delete("/organizations/<org>/tags/<id>")]
async fn remove(api: ApiUser, org: &str, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    api.require(Permission::TagsDelete)?;
    member(pool, &api, org).await?;
    tags::delete(pool, org, id).await?;
    Ok(Status::NoContent)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TagRef {
    tag_id: String,
}

/// Tag an ein Dokument hängen (Papra: `POST …/documents/<id>/tags` mit `{ "tagId" }`).
#[post("/organizations/<org>/documents/<id>/tags", data = "<body>")]
async fn add_to_document(
    api: ApiUser,
    org: &str,
    id: &str,
    body: Json<TagRef>,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    api.require(Permission::DocumentsUpdate)?;
    api.require(Permission::TagsRead)?;
    member(pool, &api, org).await?;
    let added = tags::add_to_document(pool, org, id, &body.tag_id, Some(&api.user.id)).await?;
    Ok(if added {
        Status::Created
    } else {
        Status::NoContent
    })
}

#[delete("/organizations/<org>/documents/<id>/tags/<tag>")]
async fn remove_from_document(
    api: ApiUser,
    org: &str,
    id: &str,
    tag: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    api.require(Permission::DocumentsUpdate)?;
    member(pool, &api, org).await?;
    tags::remove_from_document(pool, org, id, tag, Some(&api.user.id)).await?;
    Ok(Status::NoContent)
}

//! Quellen der Inbox: Ordner-Eingang (Administratoren richten ihn ein) und die Übersicht,
//! aus welchen Postfächern und Ordnern eine Organisation beliefert wird (für den Filter).

use crate::ai::AiConfig;
use crate::auth::guards::{AdminUser, CurrentUser};
use crate::config::Config;
use crate::error::AppResult;
use crate::services::{organizations, watch_folders};
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Route, State};
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![sources, list, create, update, remove, scan_now]
}

/// Postfächer und Ordner, die in diese Organisation liefern – nur Name und ID.
#[get("/organizations/<org>/inbox-sources")]
async fn sources(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    organizations::membership(pool, &user.0.id, org).await?;
    let mail: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, name, username FROM mail_accounts WHERE organization_id = ? ORDER BY name COLLATE NOCASE",
    )
    .bind(org)
    .fetch_all(pool.inner())
    .await?;
    let folders = watch_folders::for_org(pool, org).await?;
    Ok(json!({
        "mail": mail.into_iter().map(|(id, name, username)| json!({ "ref": format!("mail:{id}"), "name": if name.is_empty() { username } else { name } })).collect::<Vec<_>>(),
        "folders": folders.into_iter().map(|f| json!({ "ref": format!("folder:{}", f.id), "name": f.name })).collect::<Vec<_>>(),
    }))
}

#[get("/settings/watch-folders")]
async fn list(_admin: AdminUser, pool: &State<SqlitePool>) -> AppResult<Value> {
    Ok(json!({ "watchFolders": watch_folders::list(pool).await? }))
}

#[post("/settings/watch-folders", data = "<body>")]
async fn create(
    admin: AdminUser,
    body: Json<watch_folders::Input>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    organizations::membership(pool, &admin.0.id, &body.organization_id).await?;
    let f = watch_folders::create(pool, &admin.0.id, &body).await?;
    Ok((Status::Created, json!({ "watchFolder": f })))
}

#[put("/settings/watch-folders/<id>", data = "<body>")]
async fn update(
    admin: AdminUser,
    id: &str,
    body: Json<watch_folders::Input>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    organizations::membership(pool, &admin.0.id, &body.organization_id).await?;
    Ok(json!({ "watchFolder": watch_folders::update(pool, id, &body).await? }))
}

#[delete("/settings/watch-folders/<id>")]
async fn remove(_admin: AdminUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    watch_folders::delete(pool, id).await?;
    Ok(Status::NoContent)
}

/// Sofort einlesen (ohne auf das Ende des Schreibens zu warten).
#[post("/settings/watch-folders/<id>/scan")]
async fn scan_now(
    _admin: AdminUser,
    id: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let f = watch_folders::get(pool, id).await?;
    let ai = AiConfig::load(pool, config).await;
    let n = watch_folders::scan(pool, &ai, &f, &mut watch_folders::Seen::default(), false)
        .await
        .map_err(crate::error::AppError::bad)?;
    Ok(json!({ "imported": n, "watchFolder": watch_folders::get(pool, id).await? }))
}

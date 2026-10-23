//! REST-API unter `/api/v1`. Anmeldung per API-Key (`Authorization: Bearer np_…`) oder
//! Sitzung; jeder Endpunkt prüft die passende Berechtigung.

use crate::auth::guards::ApiUser;
use crate::db::models::Permission;
use crate::error::{AppError, AppResult};
use crate::services::organizations;
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{get, post, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![health, current_key, list_organizations, create_organization, get_organization]
}

/// Lebenszeichen für Docker-Healthcheck und Monitoring.
#[get("/health")]
pub fn health() -> Value {
    json!({ "status": "ok", "name": "nrzPaper", "version": env!("CARGO_PKG_VERSION") })
}

/// Wer bin ich, und was darf der Key?
#[get("/api-keys/current")]
fn current_key(api: ApiUser) -> Value {
    json!({
        "user": { "id": api.user.id, "username": api.user.username },
        "apiKey": api.key.as_ref().map(|k| json!({ "id": k.id, "name": k.name, "prefix": k.prefix, "permissions": k.permissions.0 })),
    })
}

#[get("/organizations")]
async fn list_organizations(api: ApiUser, pool: &State<SqlitePool>) -> AppResult<Value> {
    api.require(Permission::OrganizationsRead)?;
    let orgs = organizations::list_for_user(pool, &api.user.id).await?;
    Ok(json!({ "organizations": orgs.into_iter().map(|(o, role)| json!({ "id": o.id, "name": o.name, "role": role, "createdAt": o.created_at })).collect::<Vec<_>>() }))
}

#[derive(Deserialize)]
struct NewOrganization {
    name: String,
}

#[post("/organizations", data = "<body>")]
async fn create_organization(api: ApiUser, body: Json<NewOrganization>, pool: &State<SqlitePool>) -> AppResult<(Status, Value)> {
    api.require(Permission::OrganizationsCreate)?;
    let org = organizations::create(pool, &api.user.id, &body.name).await?;
    Ok((Status::Created, json!({ "organization": org })))
}

#[get("/organizations/<id>")]
async fn get_organization(api: ApiUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    api.require(Permission::OrganizationsRead)?;
    let (org, role) = organizations::membership(pool, &api.user.id, id).await.map_err(|_| AppError::NotFound)?;
    Ok(json!({ "organization": org, "role": role }))
}

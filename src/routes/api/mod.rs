//! REST-API unter `/api/v1`. Anmeldung per API-Key (`Authorization: Bearer np_…`) oder
//! Sitzung; jeder Endpunkt prüft die passende Berechtigung.

mod app;
mod documents;
mod scan;
mod tags;
mod webhooks;

use crate::auth::guards::ApiUser;
use crate::db::models::Permission;
use crate::error::{AppError, AppResult};
use crate::services::organizations;
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    let mut r = routes![
        health,
        current_key,
        list_organizations,
        create_organization,
        get_organization,
        update_organization,
        delete_organization
    ];
    r.extend(app::routes());
    r.extend(documents::routes());
    r.extend(scan::routes());
    r.extend(tags::routes());
    r.extend(webhooks::routes());
    r
}

/// Mitgliedschaft prüfen (404, wenn nicht Mitglied).
pub(crate) async fn member(
    pool: &SqlitePool,
    api: &ApiUser,
    org: &str,
) -> AppResult<crate::db::models::Role> {
    Ok(organizations::membership(pool, &api.user.id, org).await?.1)
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
    Ok(
        json!({ "organizations": orgs.into_iter().map(|(o, role)| json!({ "id": o.id, "name": o.name, "role": role, "createdAt": o.created_at })).collect::<Vec<_>>() }),
    )
}

#[derive(Deserialize)]
struct NewOrganization {
    name: String,
}

#[post("/organizations", data = "<body>")]
async fn create_organization(
    api: ApiUser,
    body: Json<NewOrganization>,
    pool: &State<SqlitePool>,
    config: &State<crate::config::Config>,
) -> AppResult<(Status, Value)> {
    api.require(Permission::OrganizationsCreate)?;
    let org = organizations::create(pool, &api.user.id, &body.name).await?;
    crate::ingestion::create_org_dir(config, &org.id).await;
    Ok((Status::Created, json!({ "organization": org })))
}

#[get("/organizations/<id>")]
async fn get_organization(api: ApiUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    api.require(Permission::OrganizationsRead)?;
    let (org, role) = organizations::membership(pool, &api.user.id, id)
        .await
        .map_err(|_| AppError::NotFound)?;
    Ok(json!({ "organization": org, "role": role }))
}

/// Organisation umbenennen (Besitzer und Admins).
#[put("/organizations/<id>", data = "<body>")]
async fn update_organization(
    api: ApiUser,
    id: &str,
    body: Json<NewOrganization>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    api.require(Permission::OrganizationsUpdate)?;
    if !organizations::can_manage(member(pool, &api, id).await?) {
        return Err(AppError::Forbidden(
            "Nur Besitzer und Admins dürfen das.".into(),
        ));
    }
    organizations::rename(pool, id, &body.name).await?;
    let (org, role) = organizations::membership(pool, &api.user.id, id).await?;
    Ok(json!({ "organization": org, "role": role }))
}

/// Organisation löschen (nur Besitzer).
#[delete("/organizations/<id>")]
async fn delete_organization(
    api: ApiUser,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    api.require(Permission::OrganizationsDelete)?;
    if member(pool, &api, id).await? != crate::db::models::Role::Owner {
        return Err(AppError::Forbidden(
            "Nur der Besitzer darf die Organisation löschen.".into(),
        ));
    }
    organizations::delete(pool, id).await?;
    Ok(Status::NoContent)
}

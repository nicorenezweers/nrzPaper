//! Webhooks über die API (`/api/v1/organizations/<org>/webhooks…`). Verwalten dürfen
//! Besitzer und Admins mit der Berechtigung `organizations:update`.

use super::member;
use crate::auth::guards::ApiUser;
use crate::db::models::{Permission, WebhookEvent};
use crate::error::{AppError, AppResult};
use crate::services::{organizations, webhooks};
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![list, create, update, remove, deliveries]
}

async fn manager(pool: &SqlitePool, api: &ApiUser, org: &str) -> AppResult<()> {
    api.require(Permission::OrganizationsUpdate)?;
    if !organizations::can_manage(member(pool, api, org).await?) {
        return Err(AppError::Forbidden(
            "Nur Besitzer und Admins dürfen das.".into(),
        ));
    }
    Ok(())
}

fn hook_json(hook: &crate::db::models::Webhook, events: &[WebhookEvent]) -> Value {
    let mut v = json!(hook);
    v["events"] = json!(events);
    v
}

#[get("/organizations/<org>/webhooks")]
async fn list(api: ApiUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    manager(pool, &api, org).await?;
    let hooks: Vec<Value> = webhooks::list(pool, org)
        .await?
        .iter()
        .map(|(h, e)| hook_json(h, e))
        .collect();
    Ok(json!({ "webhooks": hooks }))
}

#[derive(Deserialize)]
struct NewWebhook {
    name: String,
    url: String,
    secret: Option<String>,
    events: Vec<WebhookEvent>,
}

/// Neuer Webhook; das Geheimnis steht nur in dieser Antwort.
#[post("/organizations/<org>/webhooks", data = "<body>")]
async fn create(
    api: ApiUser,
    org: &str,
    body: Json<NewWebhook>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    manager(pool, &api, org).await?;
    let (hook, secret) = webhooks::create(
        pool,
        org,
        &api.user.id,
        &body.name,
        &body.url,
        body.secret.as_deref(),
        &body.events,
    )
    .await?;
    Ok((
        Status::Created,
        json!({ "webhook": hook_json(&hook, &body.events), "secret": secret }),
    ))
}

#[derive(Deserialize)]
struct WebhookUpdate {
    enabled: bool,
}

#[patch("/organizations/<org>/webhooks/<id>", data = "<body>")]
async fn update(
    api: ApiUser,
    org: &str,
    id: &str,
    body: Json<WebhookUpdate>,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    manager(pool, &api, org).await?;
    webhooks::set_enabled(pool, org, id, body.enabled).await?;
    Ok(Status::NoContent)
}

#[delete("/organizations/<org>/webhooks/<id>")]
async fn remove(api: ApiUser, org: &str, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    manager(pool, &api, org).await?;
    webhooks::delete(pool, org, id).await?;
    Ok(Status::NoContent)
}

#[get("/organizations/<org>/webhooks/<id>/deliveries")]
async fn deliveries(
    api: ApiUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    manager(pool, &api, org).await?;
    webhooks::get(pool, org, id).await?;
    Ok(json!({ "deliveries": webhooks::deliveries(pool, id, 100).await? }))
}

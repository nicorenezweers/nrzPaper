//! Netzwerkdrucker: Administratoren suchen und richten sie ein, alle Mitglieder drucken.

use crate::auth::guards::{AdminUser, CurrentUser};
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::print::{self, Options};
use crate::services::{documents, organizations};
use crate::storage;
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![list, status, discover, add, rename, remove, print_document]
}

/// Eingerichtete Drucker (für die Auswahl beim Drucken).
#[get("/printers")]
async fn list(_user: CurrentUser, pool: &State<SqlitePool>) -> AppResult<Value> {
    Ok(json!({ "printers": print::list(pool).await? }))
}

/// Zustand jetzt (bereit, druckt, angehalten) und was der Drucker kann.
#[get("/printers/<id>/status")]
async fn status(_user: CurrentUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    let p = print::get(pool, id).await?;
    Ok(match print::query(&p.url).await {
        Ok(info) => json!({ "ok": true, "info": info }),
        Err(e) => json!({ "ok": false, "error": e }),
    })
}

#[derive(Deserialize)]
struct DiscoverInput {
    #[serde(default)]
    subnets: String,
}

/// In den angegebenen oder voreingestellten /24-Netzen nach IPP-Druckern suchen.
#[post("/settings/printers/discover", data = "<body>")]
async fn discover(_admin: AdminUser, body: Json<DiscoverInput>, config: &State<Config>) -> AppResult<Value> {
    let source = if body.subnets.trim().is_empty() { config.scan_subnets.as_str() } else { body.subnets.as_str() };
    let subnets: Vec<String> = source.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let (found, searched) = print::discover(&subnets).await;
    Ok(json!({ "printers": found, "searched": searched }))
}

#[derive(Deserialize)]
struct NewPrinter {
    url: String,
    #[serde(default)]
    name: String,
}

#[post("/settings/printers", data = "<body>")]
async fn add(_admin: AdminUser, body: Json<NewPrinter>, pool: &State<SqlitePool>) -> AppResult<(Status, Value)> {
    Ok((Status::Created, json!({ "printer": print::add(pool, &body.url, &body.name).await? })))
}

#[derive(Deserialize)]
struct Rename {
    name: String,
}

#[put("/settings/printers/<id>", data = "<body>")]
async fn rename(_admin: AdminUser, id: &str, body: Json<Rename>, pool: &State<SqlitePool>) -> AppResult<Value> {
    Ok(json!({ "printer": print::rename(pool, id, &body.name).await? }))
}

#[delete("/settings/printers/<id>")]
async fn remove(_admin: AdminUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    print::remove(pool, id).await?;
    Ok(Status::NoContent)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrintInput {
    printer_id: String,
    #[serde(flatten)]
    options: Options,
}

/// Dokument auf einem eingerichteten Drucker ausgeben.
#[post("/organizations/<org>/documents/<id>/print", data = "<body>")]
async fn print_document(
    user: CurrentUser,
    org: &str,
    id: &str,
    body: Json<PrintInput>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    organizations::membership(pool, &user.0.id, org).await?;
    if !(1..=99).contains(&body.options.copies) {
        return Err(AppError::bad("Kopien: 1 bis 99."));
    }
    let doc = documents::get(pool, org, id).await?;
    let printer = print::get(pool, &body.printer_id).await?;
    let data = storage::read_all(pool, &doc.id).await?;
    let jobs = print::print_document(&printer.url, &user.0.username, &doc.name, &doc.mime_type, &data, &body.options)
        .await
        .map_err(AppError::bad)?;
    documents::log(pool, &doc.id, "printed", Some(&user.0.id), json!({ "printer": printer.name, "copies": body.options.copies }))
        .await?;
    Ok(json!({ "jobs": jobs, "printer": printer.name }))
}

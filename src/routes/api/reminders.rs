//! Erinnerungen an Dokumente (nur die eigenen). `days` statt `dueAt` setzt die Fälligkeit
//! relativ zu jetzt („in 10 Tagen“, auf 8 Uhr UTC des Tages gerundet wäre zu schlau – es zählt
//! der Zeitpunkt der Anfrage plus Tage).

use crate::auth::guards::CurrentUser;
use crate::db::models::now;
use crate::error::{AppError, AppResult};
use crate::services::{organizations, reminders};
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![list_open, for_document, create, update, remove]
}

/// Offene Erinnerungen der Organisation (fällige zuerst, dann die nächsten).
#[get("/organizations/<org>/reminders")]
async fn list_open(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    organizations::membership(pool, &user.0.id, org).await?;
    Ok(json!({ "reminders": reminders::open(pool, &user.0.id, org).await?, "now": now() }))
}

#[get("/organizations/<org>/documents/<id>/reminders")]
async fn for_document(user: CurrentUser, org: &str, id: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    organizations::membership(pool, &user.0.id, org).await?;
    Ok(json!({ "reminders": reminders::for_document(pool, &user.0.id, id).await? }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewReminder {
    /// In so vielen Tagen …
    days: Option<i64>,
    /// … oder genau zu diesem Zeitpunkt (Unix-Sekunden)
    due_at: Option<i64>,
    #[serde(default)]
    note: String,
}

#[post("/organizations/<org>/documents/<id>/reminders", data = "<body>")]
async fn create(
    user: CurrentUser,
    org: &str,
    id: &str,
    body: Json<NewReminder>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    organizations::membership(pool, &user.0.id, org).await?;
    let due = match (body.due_at, body.days) {
        (Some(at), _) => at,
        (None, Some(d)) if (0..=3650).contains(&d) => now() + d * 86_400,
        _ => return Err(AppError::bad("Bitte Tage (0–3650) oder einen Zeitpunkt angeben.")),
    };
    let r = reminders::create(pool, org, &user.0.id, id, due, &body.note).await?;
    Ok((Status::Created, json!({ "reminder": r })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReminderUpdate {
    done: Option<bool>,
    due_at: Option<i64>,
    /// Um so viele Tage ab jetzt verschieben
    snooze_days: Option<i64>,
    note: Option<String>,
}

#[put("/reminders/<id>", data = "<body>")]
async fn update(user: CurrentUser, id: &str, body: Json<ReminderUpdate>, pool: &State<SqlitePool>) -> AppResult<Value> {
    let due = body
        .due_at
        .or(body.snooze_days.filter(|d| (0..=3650).contains(d)).map(|d| now() + d * 86_400));
    // Verschieben öffnet die Erinnerung wieder
    let done = if due.is_some() && body.done.is_none() { Some(false) } else { body.done };
    let r = reminders::update(pool, &user.0.id, id, done, due, body.note.as_deref()).await?;
    Ok(json!({ "reminder": r }))
}

#[delete("/reminders/<id>")]
async fn remove(user: CurrentUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    reminders::delete(pool, &user.0.id, id).await?;
    Ok(Status::NoContent)
}

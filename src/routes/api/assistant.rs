//! Dieter, der Assistent: Fragen zu den Dokumenten einer Organisation, Antworten mit Quellen.

use crate::ai::AiConfig;
use crate::auth::guards::CurrentUser;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::services::{assistant, organizations};
use rocket::serde::json::{json, Json, Value};
use rocket::{post, routes, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![ask]
}

#[derive(Deserialize)]
struct Ask {
    messages: Vec<assistant::Message>,
}

#[post("/organizations/<org>/assistant", data = "<body>")]
async fn ask(
    user: CurrentUser,
    org: &str,
    body: Json<Ask>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    organizations::membership(pool, &user.0.id, org).await?;
    if body.messages.len() > 40 {
        return Err(AppError::bad("Der Verlauf ist zu lang – bitte ein neues Gespräch beginnen."));
    }
    let ai = AiConfig::load(pool, config).await;
    Ok(json!(assistant::ask(pool, &ai, org, &body.messages).await?))
}

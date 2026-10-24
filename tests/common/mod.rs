//! Gemeinsame Helfer der Integrationstests: Server mit leerem Datenordner, Anmeldung und
//! Organisation über die JSON-API (wie die Oberfläche), JSON-Aufrufe.
#![allow(dead_code)]

use rocket::figment::Figment;
use rocket::http::{Accept, ContentType, Status};
use rocket::local::asynchronous::Client;
use serde_json::{json, Value};
use sqlx::SqlitePool;

/// Grundkonfiguration für Tests (ohne Hintergrundaufgaben, KI ins Leere).
pub fn figment(dir: &tempfile::TempDir) -> Figment {
    nrzpaper::config::figment()
        .merge(("data_dir", dir.path()))
        .merge(("db_key", "test"))
        .merge(("log_level", "off"))
        .merge(("background", false))
        .merge(("openrouter_url", "http://127.0.0.1:9"))
}

/// Server ohne Konto.
pub async fn empty_with(adjust: impl FnOnce(Figment) -> Figment) -> (tempfile::TempDir, Client) {
    let dir = tempfile::tempdir().unwrap();
    let rocket = nrzpaper::build_with(adjust(figment(&dir))).await.unwrap();
    (dir, Client::tracked(rocket).await.unwrap())
}

pub async fn empty() -> (tempfile::TempDir, Client) {
    empty_with(|f| f).await
}

/// Server mit angemeldetem Admin „nico“ (Passwort `geheim-123`) und Organisation „Privat“.
pub async fn client_with(
    adjust: impl FnOnce(Figment) -> Figment,
) -> (tempfile::TempDir, Client, String) {
    let (dir, c) = empty_with(adjust).await;
    assert_eq!(register(&c, "nico", "geheim-123").await, Status::Created);
    let org = new_org(&c, "Privat").await;
    (dir, c, org)
}

pub async fn client() -> (tempfile::TempDir, Client, String) {
    client_with(|f| f).await
}

pub fn pool(c: &Client) -> &SqlitePool {
    c.rocket().state::<SqlitePool>().unwrap()
}

/// JSON-Aufruf mit der Sitzung des Clients.
pub async fn call(c: &Client, method: &str, uri: impl AsRef<str>, body: Option<Value>) -> (Status, Value) {
    let uri = uri.as_ref().to_string();
    let req = match method {
        "GET" => c.get(uri),
        "POST" => c.post(uri),
        "PUT" => c.put(uri),
        "PATCH" => c.patch(uri),
        "DELETE" => c.delete(uri),
        _ => unreachable!(),
    }
    .header(Accept::JSON);
    let req = match body {
        Some(b) => req.header(ContentType::JSON).body(b.to_string()),
        None => req,
    };
    let res = req.dispatch().await;
    let status = res.status();
    (status, res.into_json::<Value>().await.unwrap_or(Value::Null))
}

pub async fn register(c: &Client, user: &str, password: &str) -> Status {
    call(c, "POST", "/api/v1/auth/register", Some(json!({ "username": user, "password": password })))
        .await
        .0
}

pub async fn login(c: &Client, user: &str, password: &str) -> Status {
    call(c, "POST", "/api/v1/auth/login", Some(json!({ "username": user, "password": password })))
        .await
        .0
}

pub async fn logout(c: &Client) {
    call(c, "POST", "/api/v1/auth/logout", None).await;
}

pub async fn new_org(c: &Client, name: &str) -> String {
    let (s, v) = call(c, "POST", "/api/v1/organizations", Some(json!({ "name": name }))).await;
    assert_eq!(s, Status::Created, "{v}");
    v["organization"]["id"].as_str().unwrap().to_string()
}

/// API-Key mit den Rechten anlegen – der Key steht nur in dieser Antwort.
pub async fn create_key(c: &Client, permissions: &[&str]) -> String {
    let (s, v) = call(
        c,
        "POST",
        "/api/v1/me/api-keys",
        Some(json!({ "name": "Test", "permissions": permissions, "expiresInDays": 30 })),
    )
    .await;
    assert_eq!(s, Status::Created, "{v}");
    v["key"].as_str().unwrap().to_string()
}

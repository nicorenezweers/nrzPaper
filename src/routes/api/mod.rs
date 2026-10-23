//! REST-API unter `/api/v1`, gesichert mit API-Keys (`Authorization: Bearer …`). – Schritt 6

use rocket::serde::json::{json, Value};

/// Lebenszeichen für Docker-Healthcheck und Monitoring.
#[rocket::get("/health")]
pub fn health() -> Value {
    json!({ "status": "ok", "name": "nrzPaper", "version": env!("CARGO_PKG_VERSION") })
}

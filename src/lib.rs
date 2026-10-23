//! nrzPaper – selbst gehostete Dokumentenverwaltung. Aufbau: siehe ARCHITECTURE.md.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod fairings;
pub mod ingestion;
pub mod ocr;
pub mod routes;
pub mod search;
pub mod services;
pub mod storage;

use rocket::{Build, Rocket};

/// Baut die Rocket-Instanz: Konfiguration laden, Datenbank öffnen, Routen einhängen.
pub async fn build() -> Result<Rocket<Build>, error::AppError> {
    let figment = config::figment();
    let config: config::Config = figment.extract().map_err(|e| error::AppError::Config(e.to_string()))?;
    let pool = db::connect(&config).await?;

    Ok(rocket::custom(figment)
        .manage(pool)
        .manage(config)
        .mount("/api/v1", rocket::routes![routes::api::health]))
}

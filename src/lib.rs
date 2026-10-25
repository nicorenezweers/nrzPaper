//! nrzPaper – selbst gehostete Dokumentenverwaltung. Aufbau: siehe ARCHITECTURE.md.

pub mod ai;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod fairings;
pub mod ingestion;
pub mod ocr;
pub mod print;
pub mod routes;
pub mod scan;
pub mod search;
pub mod services;
pub mod storage;

use rocket::figment::Figment;
use rocket::{Build, Rocket};

/// Baut die Rocket-Instanz mit der normalen Konfiguration (`Rocket.toml`, `NRZPAPER_*`).
pub async fn build() -> Result<Rocket<Build>, error::AppError> {
    build_with(config::figment()).await
}

/// Baut die Rocket-Instanz aus einer gegebenen Konfiguration (auch für Tests):
/// Datenbank öffnen und migrieren, Cookie-Schlüssel laden, Fairings, Routen, Catcher.
pub async fn build_with(figment: Figment) -> Result<Rocket<Build>, error::AppError> {
    let config: config::Config = figment
        .extract()
        .map_err(|e| error::AppError::Config(e.to_string()))?;
    let pool = db::connect(&config).await?;
    // Ein selbst gesetzter secret_key hat Vorrang; sonst der aus der Datenbank.
    let figment = figment.join((
        "secret_key",
        services::settings::cookie_secret(&pool).await?,
    ));

    Ok(rocket::custom(figment)
        .manage(pool)
        .manage(config)
        .manage(ocr::OcrQueue::new())
        .attach(fairings::RequestLog)
        .attach(fairings::SecurityHeaders)
        .attach(fairings::Background)
        .attach(fairings::CloseDatabase)
        .mount("/", routes::assets::routes())
        .mount("/", routes::files::routes())
        .mount("/", routes::spa::routes())
        .mount("/api/v1", routes::api::routes())
        .register("/", routes::catchers::all()))
}

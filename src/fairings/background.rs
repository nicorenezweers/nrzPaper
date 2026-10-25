//! Startet beim Hochfahren (Liftoff) die Hintergrundarbeiten:
//! OCR-Worker, Eingangsordner, Papierkorb-Leerung (täglich) und das Aufräumen
//! abgelaufener Sitzungen und alter Einträge der Prüfliste (stündlich). Alle enden mit dem Server.

use crate::auth::session;
use crate::config::Config;
use crate::ocr::{queue, Engine, OcrQueue};
use crate::services::{documents, inbox};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Orbit, Rocket};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;

pub struct Background;

#[rocket::async_trait]
impl Fairing for Background {
    fn info(&self) -> Info {
        Info {
            name: "Hintergrundarbeiten",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let (Some(pool), Some(config), Some(ocr)) = (
            rocket.state::<SqlitePool>().cloned(),
            rocket.state::<Config>().cloned(),
            rocket.state::<OcrQueue>().cloned(),
        ) else {
            return;
        };
        if !config.background {
            return;
        }

        // Texterkennung
        match queue::recover(&pool).await {
            Ok(n) if n > 0 => tracing::info!("{n} unterbrochene Texterkennungen wieder eingereiht"),
            Err(e) => tracing::warn!("OCR-Wiederaufnahme: {e}"),
            _ => {}
        }
        let engine = Arc::new(Engine::detect(&config.ocr_languages).await);
        queue::spawn_workers(
            pool.clone(),
            engine,
            ocr.clone(),
            config.ocr_workers,
            rocket.shutdown(),
        );

        // Prüfliste: nach einem Neustart hängende Analysen/Ablagen auflösen
        match inbox::recover(&pool).await {
            Ok(n) if n > 0 => {
                tracing::info!("{n} unterbrochene Einträge der Prüfliste zurückgesetzt")
            }
            Err(e) => tracing::warn!("Prüfliste: {e}"),
            _ => {}
        }

        // Eingangsordner
        crate::ingestion::spawn(pool.clone(), ocr, &config, rocket.shutdown());

        // E-Mail-Eingang: fällige Postfächer jede Minute abholen
        crate::services::mail::spawn(pool.clone(), config.clone(), rocket.shutdown());

        // Ordner-Eingang: überwachte Ordner alle 20 Sekunden in die Inbox
        crate::services::watch_folders::spawn(pool.clone(), config.clone(), rocket.shutdown());

        // Aufräumen: Sitzungen stündlich, Papierkorb einmal am Tag (bei jedem 24. Durchlauf)
        let mut shutdown = rocket.shutdown();
        tokio::spawn(async move {
            let mut round: u64 = 0;
            loop {
                match session::purge_expired(&pool).await {
                    Ok(n) if n > 0 => tracing::info!("{n} abgelaufene Sitzungen gelöscht"),
                    Err(e) => tracing::warn!("Sitzungen aufräumen: {e}"),
                    _ => {}
                }
                match inbox::purge(&pool, config.inbox_keep_days).await {
                    Ok(n) if n > 0 => tracing::info!("{n} alte Einträge der Prüfliste gelöscht"),
                    Err(e) => tracing::warn!("Prüfliste aufräumen: {e}"),
                    _ => {}
                }
                if round % 24 == 0 {
                    match documents::purge_trash(&pool, config.trash_retention_days).await {
                        Ok(n) if n > 0 => {
                            tracing::info!("{n} Dokumente endgültig aus dem Papierkorb gelöscht")
                        }
                        Err(e) => tracing::warn!("Papierkorb leeren: {e}"),
                        _ => {}
                    }
                }
                round += 1;
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(3600)) => {}
                    _ = &mut shutdown => break,
                }
            }
        });
    }
}

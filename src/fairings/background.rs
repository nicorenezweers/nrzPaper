//! Startet beim Hochfahren (Liftoff) die Hintergrundarbeiten. Bisher: abgelaufene
//! Sitzungen stündlich löschen. In Schritt 4 kommen OCR-Warteschlange, Eingangsordner
//! und Papierkorb-Leerung dazu.

use crate::auth::session;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Orbit, Rocket};
use sqlx::SqlitePool;
use std::time::Duration;

pub struct Background;

#[rocket::async_trait]
impl Fairing for Background {
    fn info(&self) -> Info {
        Info { name: "Hintergrundarbeiten", kind: Kind::Liftoff }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let Some(pool) = rocket.state::<SqlitePool>().cloned() else { return };
        let mut shutdown = rocket.shutdown();
        tokio::spawn(async move {
            loop {
                match session::purge_expired(&pool).await {
                    Ok(n) if n > 0 => tracing::info!("{n} abgelaufene Sitzungen gelöscht"),
                    Err(e) => tracing::warn!("Sitzungen aufräumen: {e}"),
                    _ => {}
                }
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(3600)) => {}
                    _ = &mut shutdown => break,
                }
            }
        });
    }
}

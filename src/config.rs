//! Einstellungen. Quelle ist Rockets Figment: `Rocket.toml` und Umgebungsvariablen mit
//! dem Präfix `NRZPAPER_` (z. B. `NRZPAPER_DB_KEY`, `NRZPAPER_DATA_DIR`).

use rocket::figment::providers::{Env, Format, Toml};
use rocket::figment::Figment;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Datenordner: Datenbank, Schlüsseldatei, Eingangsordner.
    pub data_dir: PathBuf,
    /// Schlüssel der Datenbank. Fehlt er, wird `data_dir/nrzpaper.key` erzeugt bzw. gelesen.
    pub db_key: Option<String>,
    /// Tage, die gelöschte Dokumente im Papierkorb bleiben.
    pub trash_retention_days: u32,
    /// OCR-Sprachen für Tesseract, z. B. "deu+eng".
    pub ocr_languages: String,
    /// Gleichzeitige OCR-Aufträge.
    pub ocr_workers: usize,
    /// Neue Benutzer dürfen sich selbst registrieren (das erste Konto immer).
    pub registration_open: bool,
    /// Cookies nur über HTTPS senden – einschalten, sobald nrzPaper hinter HTTPS läuft.
    pub cookie_secure: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            db_key: None,
            trash_retention_days: 30,
            ocr_languages: "deu+eng".into(),
            ocr_workers: 2,
            registration_open: false,
            cookie_secure: false,
        }
    }
}

impl Config {
    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("nrzpaper.db")
    }

    pub fn key_path(&self) -> PathBuf {
        self.data_dir.join("nrzpaper.key")
    }

    pub fn ingestion_dir(&self) -> PathBuf {
        self.data_dir.join("ingestion")
    }
}

/// Rocket-Konfiguration plus `NRZPAPER_*`-Umgebung (statt `ROCKET_*`).
pub fn figment() -> Figment {
    Figment::from(rocket::Config::default())
        .merge(Toml::file(Env::var_or("NRZPAPER_CONFIG", "Rocket.toml")).nested())
        .merge(Env::prefixed("NRZPAPER_").ignore(&["PROFILE"]).global())
        .select(rocket::figment::Profile::from_env_or("NRZPAPER_PROFILE", rocket::Config::DEFAULT_PROFILE))
}

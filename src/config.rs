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
    /// Hintergrundarbeiten (OCR, Eingangsordner, Aufräumen) starten. Tests schalten sie aus.
    pub background: bool,
    /// Voreingestellter Scanner (eSCL/AirScan) für neue Benutzer, z. B. "http://192.168.1.50".
    pub scanner_url: String,
    /// Netze, in denen "Scanner suchen" sucht (Komma-getrennt, je /24).
    pub scan_subnets: String,
    /// OpenRouter-Schlüssel; fehlt er, gilt der beim Bauen eingebaute (NRZ_OPENROUTER_KEY).
    pub openrouter_key: Option<String>,
    pub openrouter_model: String,
    /// Basis-Adresse der OpenRouter-API (Tests setzen einen eigenen Server).
    pub openrouter_url: String,
    /// So viele Tage bleiben nicht freigegebene Einträge der Prüfliste liegen.
    pub inbox_keep_days: u32,
    /// KI-Anbieter aus der Umgebung (z. B. "openai", "anthropic", "ollama"; leer = OpenRouter).
    /// In den Einstellungen der Oberfläche lässt sich jeder Anbieter wählen.
    pub ai_provider: String,
    /// Schlüssel für `ai_provider`
    pub ai_key: Option<String>,
    /// Eigene Adresse der API (leer = Vorgabe des Anbieters)
    pub ai_url: String,
    /// Modell (leer = Vorgabe des Anbieters)
    pub ai_model: String,
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
            background: true,
            scanner_url: String::new(),
            scan_subnets: "192.168.0.0/24,192.168.1.0/24".into(),
            openrouter_key: None,
            openrouter_model: "openrouter/free".into(),
            openrouter_url: "https://openrouter.ai/api/v1".into(),
            inbox_keep_days: 7,
            ai_provider: String::new(),
            ai_key: None,
            ai_url: String::new(),
            ai_model: String::new(),
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
        // NRZPAPER_LOG ist der tracing-Filter (main.rs), keine Rocket-Einstellung
        .merge(
            Env::prefixed("NRZPAPER_")
                .ignore(&["PROFILE", "LOG"])
                .global(),
        )
        .select(rocket::figment::Profile::from_env_or(
            "NRZPAPER_PROFILE",
            rocket::Config::DEFAULT_PROFILE,
        ))
}

//! Ein Fehlertyp für die ganze Anwendung. In Schritt 3 bekommt er einen Rocket-`Responder`,
//! der je nach Anfrage eine HTML-Fehlerseite oder JSON liefert.

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Konfiguration: {0}")]
    Config(String),
    #[error("Datenbank: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Datei: {0}")]
    Io(#[from] std::io::Error),
}

//! Ein Fehlertyp für die ganze Anwendung. Als Rocket-`Responder` liefert er für die API
//! (`/api/…` oder `Accept: application/json`) JSON, sonst eine HTML-Fehlerseite.
//! Interne Fehler (Datenbank, Dateien) werden protokolliert, aber nicht nach aussen gezeigt.

use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use std::io::Cursor;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Konfiguration: {0}")]
    Config(String),
    #[error("Datenbank: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Datei: {0}")]
    Io(#[from] std::io::Error),
    #[error("Darstellung: {0}")]
    Template(#[from] askama::Error),
    #[error("{0}")]
    BadRequest(String),
    #[error("Bitte anmelden.")]
    Unauthorized,
    #[error("{0}")]
    Forbidden(String),
    #[error("Nicht gefunden.")]
    NotFound,
    #[error("{0}")]
    Conflict(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn bad(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        AppError::Forbidden(msg.into())
    }

    pub fn status(&self) -> Status {
        match self {
            AppError::BadRequest(_) => Status::BadRequest,
            AppError::Unauthorized => Status::Unauthorized,
            AppError::Forbidden(_) => Status::Forbidden,
            AppError::NotFound => Status::NotFound,
            AppError::Conflict(_) => Status::Conflict,
            AppError::Database(sqlx::Error::RowNotFound) => Status::NotFound,
            AppError::Config(_) | AppError::Database(_) | AppError::Io(_) | AppError::Template(_) => Status::InternalServerError,
        }
    }

    /// Kurzer, maschinenlesbarer Code für die API.
    pub fn code(&self) -> &'static str {
        code_for(self.status())
    }

    /// Was der Benutzer sehen darf – interne Details bleiben im Protokoll.
    pub fn public_message(&self) -> String {
        if self.status().code >= 500 {
            "Interner Fehler – Details stehen im Server-Protokoll.".into()
        } else if matches!(self, AppError::Database(sqlx::Error::RowNotFound)) {
            "Nicht gefunden.".into()
        } else {
            self.to_string()
        }
    }

    /// Verstoss gegen eine UNIQUE-Bedingung als verständlichen Konflikt melden.
    pub fn unique(e: sqlx::Error, msg: &str) -> Self {
        match &e {
            sqlx::Error::Database(db) if db.is_unique_violation() => AppError::Conflict(msg.into()),
            _ => AppError::Database(e),
        }
    }
}

/// Soll die Antwort JSON sein? API-Pfade immer, sonst nach `Accept`.
pub fn wants_json(req: &Request<'_>) -> bool {
    req.uri().path().starts_with("/api/") || req.accept().is_some_and(|a| a.preferred().is_json())
}

/// Antwort für einen Status mit Meldung – genutzt vom Responder und von den Catchern.
pub fn error_response(req: &Request<'_>, status: Status, message: &str) -> response::Result<'static> {
    let (content_type, body) = if wants_json(req) {
        (ContentType::JSON, serde_json::json!({ "error": { "code": code_for(status), "message": message } }).to_string())
    } else {
        let page = crate::templates::ErrorPage { status: status.code, title: status.reason_lossy(), message };
        let html = askama::Template::render(&page).unwrap_or_else(|_| format!("<h1>{}</h1><p>{}</p>", status.code, message));
        (ContentType::HTML, html)
    };
    Response::build().status(status).header(content_type).sized_body(body.len(), Cursor::new(body)).ok()
}

/// Maschinenlesbarer Code zu einem HTTP-Status.
pub fn code_for(status: Status) -> &'static str {
    match status.code {
        400 => "bad_request",
        401 => "unauthorized",
        403 => "forbidden",
        404 => "not_found",
        409 => "conflict",
        413 => "payload_too_large",
        422 => "unprocessable",
        _ => "internal_error",
    }
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status();
        if status.code >= 500 {
            tracing::error!(path = %req.uri(), "{self}");
        }
        error_response(req, status, &self.public_message())
    }
}

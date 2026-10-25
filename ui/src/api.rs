//! Zugriff auf die JSON-API unter `/api/v1`. Anmeldung läuft über das Sitzungs-Cookie
//! (gleiche Herkunft); Fehler kommen als `{ "error": { "message" } }`.

use gloo_net::http::{Request, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

pub const BASE: &str = "/api/v1";

#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

fn tr_f(text: &str, arg: &str) -> String {
    crate::i18n::trf(text, &[&arg])
}

fn network(e: impl std::fmt::Display) -> ApiError {
    ApiError { status: 0, message: tr_f("Keine Verbindung zum Server ({0}).", &e.to_string()) }
}

async fn finish<T: DeserializeOwned>(res: Response) -> ApiResult<T> {
    let status = res.status();
    if status == 204 {
        return serde_json::from_value(Value::Null).map_err(|e| ApiError { status, message: e.to_string() });
    }
    let text = res.text().await.map_err(network)?;
    if (200..300).contains(&status) {
        let value: Value = if text.trim().is_empty() { Value::Null } else { serde_json::from_str(&text).map_err(|e| ApiError { status, message: e.to_string() })? };
        return serde_json::from_value(value).map_err(|e| ApiError { status, message: tr_f("Unerwartete Antwort: {0}", &e.to_string()) });
    }
    let message = serde_json::from_str::<Value>(&text)
        .ok()
        // Meldungen des Servers sind deutsch – gibt es sie im Katalog, erscheinen sie übersetzt
        .and_then(|v| v["error"]["message"].as_str().map(crate::i18n::tr))
        .unwrap_or_else(|| match status {
            401 => crate::i18n::tr("Bitte anmelden."),
            403 => crate::i18n::tr("Keine Berechtigung."),
            404 => crate::i18n::tr("Nicht gefunden."),
            _ => tr_f("Fehler {0}", &status.to_string()),
        });
    if status == 401 {
        crate::state::on_unauthorized();
    }
    Err(ApiError { status, message })
}

fn builder(method: &str, path: &str) -> RequestBuilder {
    let url = if path.starts_with('/') && !path.starts_with(BASE) && !path.starts_with("/static") { format!("{BASE}{path}") } else { path.to_string() };
    let b = match method {
        "POST" => Request::post(&url),
        "PUT" => Request::put(&url),
        "PATCH" => Request::patch(&url),
        "DELETE" => Request::delete(&url),
        _ => Request::get(&url),
    };
    b.header("Accept", "application/json")
}

pub async fn get<T: DeserializeOwned>(path: &str) -> ApiResult<T> {
    finish(builder("GET", path).send().await.map_err(network)?).await
}

pub async fn send<T: DeserializeOwned, B: Serialize>(method: &str, path: &str, body: &B) -> ApiResult<T> {
    let req = builder(method, path).json(body).map_err(network)?;
    finish(req.send().await.map_err(network)?).await
}

pub async fn post<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> ApiResult<T> {
    send("POST", path, body).await
}

pub async fn put<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> ApiResult<T> {
    send("PUT", path, body).await
}

pub async fn patch<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> ApiResult<T> {
    send("PATCH", path, body).await
}

/// POST ohne Inhalt (Aktionen wie „neu analysieren“).
pub async fn action<T: DeserializeOwned>(path: &str) -> ApiResult<T> {
    finish(builder("POST", path).send().await.map_err(network)?).await
}

pub async fn delete(path: &str) -> ApiResult<()> {
    let _: Value = finish(builder("DELETE", path).send().await.map_err(network)?).await?;
    Ok(())
}

/// Dateien hochladen (multipart, Feld `file`).
pub async fn upload<T: DeserializeOwned>(path: &str, files: &[web_sys::File]) -> ApiResult<T> {
    let form = web_sys::FormData::new().map_err(|_| network("FormData"))?;
    for f in files {
        form.append_with_blob_and_filename("file", f, &f.name()).map_err(|_| network("FormData"))?;
    }
    let req = builder("POST", path).body(form).map_err(network)?;
    finish(req.send().await.map_err(network)?).await
}

/// URL-Kodierung für Pfad- und Suchteile.
pub fn enc(s: &str) -> String {
    urlencoding::encode(s).into_owned()
}

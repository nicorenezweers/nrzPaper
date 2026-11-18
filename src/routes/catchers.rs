//! Fehler, die vor einem Handler entstehen (Guard abgewiesen, Route fehlt, Formular kaputt).
//! Die API bekommt JSON, die Oberfläche eine Seite – und bei 401 die Anmeldung.

use crate::error::{error_response, wants_json};
use rocket::http::Status;
use rocket::response::{self, Redirect, Responder};
use rocket::{catch, catchers, Catcher, Request};

pub fn all() -> Vec<Catcher> {
    catchers![unauthorized, default]
}

pub struct Reply(Status, &'static str);

impl<'r> Responder<'r, 'static> for Reply {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        error_response(req, self.0, self.1)
    }
}

fn message(status: Status) -> &'static str {
    match status.code {
        400 => "Die Anfrage ist ungültig.",
        401 => "Bitte anmelden.",
        403 => "Kein Zugriff.",
        404 => "Diese Seite gibt es nicht.",
        413 => "Die Datei ist zu gross.",
        422 => "Die Eingaben sind unvollständig oder ungültig.",
        _ => "Interner Fehler – Details stehen im Server-Protokoll.",
    }
}

#[derive(rocket::Responder)]
pub enum UnauthorizedReply {
    Login(Redirect),
    Error(Reply),
}

/// Nicht angemeldet: Oberfläche → zur Anmeldung (mit Rücksprung), API → 401 JSON.
#[catch(401)]
fn unauthorized(req: &Request<'_>) -> UnauthorizedReply {
    if wants_json(req) {
        return UnauthorizedReply::Error(Reply(
            Status::Unauthorized,
            message(Status::Unauthorized),
        ));
    }
    let next = req.uri().to_string();
    let target = if next == "/" {
        "/login".to_string()
    } else {
        format!("/login?next={}", urlencode(&next))
    };
    UnauthorizedReply::Login(Redirect::to(target))
}

#[catch(default)]
fn default(status: Status, _req: &Request<'_>) -> Reply {
    Reply(status, message(status))
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

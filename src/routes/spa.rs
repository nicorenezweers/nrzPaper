//! Die Oberfläche: eine Dioxus-App (WebAssembly, Quelltext in `ui/`), beim Übersetzen
//! eingebettet (siehe build.rs). Ihre Dateien liegen unter `/assets/`; jeder andere Pfad,
//! der nicht zur API gehört, bekommt `index.html` – den Rest erledigt der Router der App.

use rocket::http::{ContentType, Header, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::{self, Responder, Response};
use rocket::{get, routes, Request, Route};
use std::io::Cursor;
use std::path::PathBuf;

pub struct SpaFile {
    pub path: &'static str,
    pub body: &'static [u8],
    pub gzip: Option<&'static [u8]>,
}

include!(concat!(env!("OUT_DIR"), "/spa.rs"));

pub fn routes() -> Vec<Route> {
    routes![index, fallback]
}

/// Ist die Oberfläche eingebettet? (Ohne `scripts/ui.sh` vor dem Übersetzen nicht.)
pub fn available() -> bool {
    FILES.iter().any(|f| f.path == "index.html")
}

fn find(path: &str) -> Option<&'static SpaFile> {
    FILES.iter().find(|f| f.path == path)
}

/// Versteht der Browser gzip?
pub struct AcceptsGzip(bool);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AcceptsGzip {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, ()> {
        let gzip = req
            .headers()
            .get("Accept-Encoding")
            .any(|v| v.split(',').any(|e| e.trim().starts_with("gzip")));
        Outcome::Success(AcceptsGzip(gzip))
    }
}

pub enum Reply {
    File(&'static SpaFile, bool),
    /// Freigabeseite: wie `File`, aber nicht für Suchmaschinen
    Shared(&'static SpaFile, bool),
    Missing,
}

impl<'r> Responder<'r, 'static> for Reply {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (file, gzip, noindex) = match self {
            Reply::File(f, g) => (f, g, false),
            Reply::Shared(f, g) => (f, g, true),
            Reply::Missing => {
                let body = "<!doctype html><meta charset=utf-8><title>nrzPaper</title>\
                    <p style=\"font-family:sans-serif\">Die Oberfläche ist nicht eingebaut – vor dem \
                    Übersetzen <code>scripts/ui.sh</code> ausführen. Die API unter /api/v1 läuft.</p>";
                return Response::build()
                    .status(Status::ServiceUnavailable)
                    .header(ContentType::HTML)
                    .sized_body(body.len(), Cursor::new(body))
                    .ok();
            }
        };
        let ext = file.path.rsplit('.').next().unwrap_or("");
        let content_type = ContentType::from_extension(ext).unwrap_or(ContentType::Binary);
        // Dateien unter assets/ tragen eine Prüfsumme im Namen; index.html nie zwischenspeichern
        let cache = if file.path.starts_with("assets/") {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        };
        let mut res = Response::build();
        res.header(content_type)
            .header(Header::new("Cache-Control", cache))
            .header(Header::new("Vary", "Accept-Encoding"));
        if noindex {
            res.header(Header::new("X-Robots-Tag", "noindex"));
        }
        match (gzip, file.gzip) {
            (true, Some(gz)) => res
                .header(Header::new("Content-Encoding", "gzip"))
                .sized_body(gz.len(), Cursor::new(gz)),
            _ => res.sized_body(file.body.len(), Cursor::new(file.body)),
        };
        res.ok()
    }
}

fn page(gzip: AcceptsGzip) -> Reply {
    find("index.html").map_or(Reply::Missing, |f| Reply::File(f, gzip.0))
}

#[get("/")]
fn index(gzip: AcceptsGzip) -> Reply {
    page(gzip)
}

/// Dateien der App oder – für alle Seiten der App – index.html. Niedriger Rang, damit API,
/// `/static`, Freigaben usw. zuerst greifen; unbekannte API-Pfade bleiben 404 (JSON).
#[get("/<path..>", rank = 50)]
fn fallback(path: PathBuf, gzip: AcceptsGzip) -> Option<Reply> {
    let path = path.to_str()?.replace('\\', "/");
    if path.starts_with("api/") || path == "api" {
        return None;
    }
    if let Some(f) = find(&path) {
        return Some(Reply::File(f, gzip.0));
    }
    // Fehlende Datei (z. B. alte Prüfsumme nach einem Update) nicht als Seite beantworten
    if path.starts_with("assets/") || path.starts_with("static/") {
        return None;
    }
    match page(gzip) {
        Reply::File(f, g) if path.starts_with("share/") => Some(Reply::Shared(f, g)),
        other => Some(other),
    }
}

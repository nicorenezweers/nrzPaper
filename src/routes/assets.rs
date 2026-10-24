//! Statische Dateien, ins Programm eingebettet – nrzPaper bleibt ein einzelnes Programm
//! und die PDF-Vorschau funktioniert auch ohne Internet.

use rocket::http::{ContentType, Header};
use rocket::response::{self, Responder, Response};
use rocket::{get, routes, Request, Route};
use std::io::Cursor;

pub fn routes() -> Vec<Route> {
    routes![asset, favicon]
}

/// Prüfsumme der eigenen Dateien – hängt als `?v=` an den Links. Ändert sich eine Datei,
/// ändert sich die Adresse, und Browser laden neu (statt eine Stunde die alte Fassung zu nutzen).
pub static VERSION: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for path in ["preview.js", "viewer.js", "viewer.css"] {
        h.update(lookup(path).map(|a| a.body).unwrap_or_default());
    }
    hex::encode(h.finalize())[..12].to_string()
});

struct Asset {
    body: &'static [u8],
    content_type: ContentType,
    immutable: bool,
}

fn lookup(path: &str) -> Option<Asset> {
    let js = ContentType::JavaScript;
    let (body, content_type, immutable): (&'static [u8], ContentType, bool) = match path {
        "preview.js" => (include_bytes!("../../static/preview.js"), js, false),
        "viewer.js" => (include_bytes!("../../static/viewer.js"), js, false),
        "viewer.css" => (
            include_bytes!("../../static/viewer.css"),
            ContentType::CSS,
            false,
        ),
        "vendor/pdfjs/pdf.min.mjs" => (
            include_bytes!("../../static/vendor/pdfjs/pdf.min.mjs"),
            js,
            true,
        ),
        "vendor/pdfjs/pdf.worker.min.mjs" => (
            include_bytes!("../../static/vendor/pdfjs/pdf.worker.min.mjs"),
            js,
            true,
        ),
        "fonts/inter-latin.woff2" => (
            include_bytes!("../../static/fonts/inter-latin.woff2"),
            ContentType::WOFF2,
            true,
        ),
        "fonts/inter-latin-ext.woff2" => (
            include_bytes!("../../static/fonts/inter-latin-ext.woff2"),
            ContentType::WOFF2,
            true,
        ),
        "icon.png" => (
            include_bytes!("../../static/icon.png"),
            ContentType::PNG,
            false,
        ),
        "favicon-32.png" => (
            include_bytes!("../../static/favicon-32.png"),
            ContentType::PNG,
            false,
        ),
        _ => return None,
    };
    Some(Asset {
        body,
        content_type,
        immutable,
    })
}

impl<'r> Responder<'r, 'static> for Asset {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let cache = if self.immutable {
            "public, max-age=31536000, immutable"
        } else {
            "public, max-age=3600"
        };
        Response::build()
            .header(self.content_type)
            .header(Header::new("Cache-Control", cache))
            .sized_body(self.body.len(), Cursor::new(self.body))
            .ok()
    }
}

#[get("/static/<path..>")]
fn asset(path: std::path::PathBuf) -> Option<Asset> {
    lookup(path.to_str()?)
}

/// Browser fragen ohne Link nach /favicon.ico – dasselbe Icon wie die Umbrel-App.
#[get("/favicon.ico")]
fn favicon() -> Option<Asset> {
    lookup("favicon-32.png")
}

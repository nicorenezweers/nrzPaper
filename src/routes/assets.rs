//! Statische Dateien, ins Programm eingebettet – nrzPaper bleibt ein einzelnes Programm
//! und die PDF-Vorschau funktioniert auch ohne Internet.

use rocket::http::{ContentType, Header};
use rocket::response::{self, Responder, Response};
use rocket::{get, routes, Request, Route};
use std::io::Cursor;

pub fn routes() -> Vec<Route> {
    routes![asset, favicon, manifest, service_worker, share_target]
}

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

/// Übersetzung der Oberfläche (deutsch → Sprache), z. B. `locales/fr.json`.
fn locale(path: &str) -> Option<Asset> {
    let lang = path.strip_prefix("locales/")?.strip_suffix(".json")?;
    LOCALES.iter().find(|(l, _)| *l == lang).map(|(_, body)| Asset {
        body,
        content_type: ContentType::JSON,
        immutable: false,
    })
}

/// Prüfsumme der eigenen Dateien – hängt als `?v=` an den Links. Ändert sich eine Datei,
/// ändert sich die Adresse, und Browser laden neu (statt eine Stunde die alte Fassung zu nutzen).
pub static VERSION: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for path in ["preview.js", "viewer.js", "viewer.css", "mobile.js"] {
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
        "icon-180.png" => (include_bytes!("../../static/icon-180.png"), ContentType::PNG, false),
        "icon-192.png" => (include_bytes!("../../static/icon-192.png"), ContentType::PNG, false),
        "icon-512.png" => (include_bytes!("../../static/icon-512.png"), ContentType::PNG, false),
        "mobile.js" => (include_bytes!("../../static/mobile.js"), js, false),
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
    let path = path.to_str()?;
    lookup(path).or_else(|| locale(path))
}

/// Browser fragen ohne Link nach /favicon.ico – dasselbe Icon wie die Umbrel-App.
#[get("/favicon.ico")]
fn favicon() -> Option<Asset> {
    lookup("favicon-32.png")
}

/// Web-App-Manifest: „Zum Home-Bildschirm“ (iPhone) bzw. „App installieren“ (Android).
/// `share_target` lässt nrzPaper auf Android im Teilen-Menü erscheinen; das iPhone kennt
/// das für Web-Apps nicht – dort hilft ein Kurzbefehl (Einstellungen → Handy & Teilen).
#[get("/manifest.webmanifest")]
fn manifest() -> NoCache {
    NoCache {
        body: include_str!("../../static/manifest.webmanifest"),
        content_type: ContentType::new("application", "manifest+json"),
        cache: Header::new("Cache-Control", "no-cache"),
    }
}

/// Immer frisch vom Server (Manifest und Service Worker ändern sich mit Updates).
#[derive(rocket::Responder)]
struct NoCache {
    body: &'static str,
    content_type: ContentType,
    cache: Header<'static>,
}

/// Service Worker – muss unter `/` liegen, damit er das Teilen-Ziel `/share-target` sieht.
#[get("/sw.js")]
fn service_worker() -> NoCache {
    NoCache {
        body: include_str!("../../static/sw.js"),
        content_type: ContentType::JavaScript,
        cache: Header::new("Cache-Control", "no-cache"),
    }
}

/// Teilen-Ziel ohne Service Worker (noch nicht installiert): zurück zur App mit Hinweis.
/// Mit Service Worker kommt die Anfrage gar nicht hier an.
#[rocket::post("/share-target")]
fn share_target() -> rocket::response::Redirect {
    rocket::response::Redirect::to("/?shared=failed")
}

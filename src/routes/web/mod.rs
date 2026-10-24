//! Server-seitig gerenderte Seiten.

pub use documents::{serve as serve_file, FileReply};

mod auth;
mod documents;
mod organizations;
mod pages;

use rocket::response::content::RawHtml;
use rocket::response::status::Custom;
use rocket::response::Redirect;
use rocket::Route;

pub fn routes() -> Vec<Route> {
    let mut r = auth::routes();
    r.extend(organizations::routes());
    r.extend(documents::routes());
    r.extend(pages::routes());
    r
}

/// Antwort einer Seite: HTML, HTML mit Status (z. B. falsches Passwort) oder Weiterleitung.
#[derive(rocket::Responder)]
pub enum Page {
    Html(RawHtml<String>),
    Status(Custom<RawHtml<String>>),
    Redirect(Redirect),
}

/// Nur Rücksprünge innerhalb der eigenen Seite zulassen (kein offener Redirect).
pub fn safe_next(next: Option<&str>) -> String {
    match next {
        Some(n) if n.starts_with('/') && !n.starts_with("//") && !n.contains('\\') => n.to_string(),
        _ => "/".to_string(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn next_only_local() {
        assert_eq!(
            super::safe_next(Some("/organizations/x")),
            "/organizations/x"
        );
        assert_eq!(super::safe_next(Some("//evil.example")), "/");
        assert_eq!(super::safe_next(Some("https://evil.example")), "/");
        assert_eq!(super::safe_next(Some("/\\evil")), "/");
        assert_eq!(super::safe_next(None), "/");
    }
}

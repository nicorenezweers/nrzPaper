//! Globaler Zustand: angemeldeter Benutzer, Überblick der geöffneten Organisation, Meldungen.

use crate::models::{Me, Overview};
use dioxus::prelude::*;

/// Angemeldeter Benutzer mit seinen Organisationen (None = noch nicht geladen / abgemeldet).
pub static ME: GlobalSignal<Option<Me>> = Signal::global(|| None);

/// Überblick (Tags, Ordner, Inbox-Zähler) der geöffneten Organisation.
pub static OVERVIEW: GlobalSignal<Option<(String, Overview)>> = Signal::global(|| None);

/// Erhöhen, damit Seitenleiste und Listen neu laden (nach Änderungen).
pub static REFRESH: GlobalSignal<u64> = Signal::global(|| 0);

pub fn refresh() {
    *REFRESH.write() += 1;
}

/// Nach 401: zur Anmeldung (ausser auf öffentlichen Seiten).
pub fn on_unauthorized() {
    let Some(window) = web_sys::window() else { return };
    let path = window.location().pathname().unwrap_or_default();
    if path.starts_with("/login") || path.starts_with("/setup") || path.starts_with("/register") || path.starts_with("/share/") {
        return;
    }
    *ME.write() = None;
    let next = crate::api::enc(&path);
    let _ = window.location().assign(&format!("/login?next={next}"));
}

/// Überblick der Organisation neu laden.
pub async fn load_overview(org: &str) {
    if let Ok(o) = crate::api::get::<Overview>(&format!("/organizations/{org}/overview")).await {
        *OVERVIEW.write() = Some((org.to_string(), o));
    }
}

pub async fn load_me() -> Option<Me> {
    match crate::api::get::<Me>("/me").await {
        Ok(me) => {
            *ME.write() = Some(me.clone());
            Some(me)
        }
        Err(_) => None,
    }
}

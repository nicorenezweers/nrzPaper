//! Übersetzungen. Die deutschen Texte im Quelltext sind zugleich die Schlüssel (wie gettext):
//! `tr("Papierkorb")`, mit Platzhaltern `trf("seit {0}", &[&datum])`. Die Kataloge liegen als
//! `locales/<sprache>.json` beim Server (deutsch → Sprache) und werden bei Bedarf geladen –
//! die App selbst wird dadurch nicht grösser. Fehlt ein Text, erscheint der deutsche.
//! Meldungen des Servers sind ebenfalls deutsch und laufen durch denselben Katalog.

use dioxus::prelude::*;
use std::collections::HashMap;
use std::fmt::Display;

/// Unterstützte Sprachen: Code und Name in der Sprache selbst. Eine weitere Sprache kommt
/// dazu mit `locales/<code>.json` (alle Texte aus `locales/strings.json`) und einer Zeile hier.
pub const LANGUAGES: &[(&str, &str)] = &[
    ("de", "Deutsch"),
    ("en", "English"),
    ("fr", "Français"),
    ("it", "Italiano"),
    ("es", "Español"),
    ("pt", "Português"),
    ("nl", "Nederlands"),
];

/// Aktuelle Sprache (Code aus `LANGUAGES`).
pub static LANG: GlobalSignal<String> = Signal::global(|| "de".to_string());
/// Katalog der aktuellen Sprache (leer bei Deutsch).
static CATALOG: GlobalSignal<HashMap<String, String>> = Signal::global(HashMap::new);

const STORAGE_KEY: &str = "nrz-lang";

/// Text in der aktuellen Sprache.
pub fn tr(source: &str) -> String {
    if LANG.read().as_str() == "de" {
        return source.to_string();
    }
    CATALOG
        .read()
        .get(source)
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| source.to_string())
}

/// Text mit Platzhaltern `{0}`, `{1}`, … in der aktuellen Sprache.
pub fn trf(source: &str, args: &[&dyn Display]) -> String {
    fill(&tr(source), args)
}

fn fill(text: &str, args: &[&dyn Display]) -> String {
    let mut out = text.replace("{{", "\u{1}").replace("}}", "\u{2}");
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), &a.to_string());
    }
    out.replace('\u{1}', "{").replace('\u{2}', "}")
}

/// Sprache für Datumsangaben (`Intl`/`toLocaleDateString`).
pub fn locale() -> &'static str {
    match LANG.read().as_str() {
        "de" => "de-CH",
        "en" => "en-GB",
        "fr" => "fr-CH",
        "it" => "it-CH",
        "nb" => "nb-NO",
        "zh" => "zh-CN",
        other => LANGUAGES.iter().find(|(c, _)| *c == other).map(|(c, _)| *c).unwrap_or("de-CH"),
    }
}

/// Gespeicherte oder vom Browser gewünschte Sprache.
fn preferred() -> String {
    if let Some(saved) = crate::util::storage_get(STORAGE_KEY) {
        if LANGUAGES.iter().any(|(c, _)| *c == saved) {
            return saved;
        }
    }
    let wanted: Vec<String> = web_sys::window()
        .map(|w| {
            let n = w.navigator();
            let mut v: Vec<String> = n.languages().iter().filter_map(|l| l.as_string()).collect();
            if let Some(l) = n.language() {
                v.push(l);
            }
            v
        })
        .unwrap_or_default();
    for w in wanted {
        let w = w.to_lowercase();
        let base = w.split('-').next().unwrap_or("");
        let code = match base {
            "no" | "nn" => "nb",
            other => other,
        };
        if let Some((c, _)) = LANGUAGES.iter().find(|(c, _)| *c == code) {
            return c.to_string();
        }
    }
    "de".into()
}

/// Beim Start: Sprache bestimmen und ihren Katalog laden.
pub async fn init() {
    set_language(&preferred(), false).await;
}

/// Sprache wechseln (und merken).
pub async fn set_language(code: &str, remember: bool) {
    let code = LANGUAGES.iter().find(|(c, _)| *c == code).map(|(c, _)| *c).unwrap_or("de");
    if remember {
        crate::util::storage_set(STORAGE_KEY, code);
    }
    if code != "de" {
        let url = format!("/static/locales/{code}.json?v={}", env!("CARGO_PKG_VERSION"));
        match gloo_net::http::Request::get(&url).send().await {
            Ok(res) if res.ok() => {
                if let Ok(map) = res.json::<HashMap<String, String>>().await {
                    *CATALOG.write() = map;
                }
            }
            _ => {
                // Ohne Katalog bleibt es deutsch – lieber verständlich als halb übersetzt
                *CATALOG.write() = HashMap::new();
                *LANG.write() = "de".into();
                set_document_lang("de");
                return;
            }
        }
    } else {
        *CATALOG.write() = HashMap::new();
    }
    *LANG.write() = code.to_string();
    set_document_lang(code);
}

fn set_document_lang(code: &str) {
    if let Some(el) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()) {
        let _ = el.set_attribute("lang", code);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fills_placeholders() {
        assert_eq!(super::fill("„{0}“ von {1} – {{x}}", &[&"A", &3]), "„A“ von 3 – {x}");
    }
}

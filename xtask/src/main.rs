//! Hilfsaufgaben rund um den Build:
//!
//!     cargo xtask ui            Oberfläche (Dioxus, ui/) bauen und nach ui/dist/public legen
//!     cargo xtask ui-files      nur ui/index.html und ui/tailwind.css schreiben (für `dx serve`)
//!     cargo xtask i18n          locales/strings.json neu schreiben, Stand je Sprache zeigen
//!     cargo xtask i18n --check  nur prüfen (Exit 1, wenn eine Sprache unvollständig ist)

mod i18n;
mod ui;

use std::path::PathBuf;
use std::process::ExitCode;

/// Wurzel des Projekts (eine Ebene über xtask/).
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("ui") => ui::build(),
        Some("ui-files") => ui::write_files().map(|()| ExitCode::SUCCESS),
        Some("i18n") => i18n::run(args.iter().any(|a| a == "--check")),
        _ => {
            eprintln!("Aufgaben: ui | ui-files | i18n [--check]");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Fehler: {e}");
            ExitCode::FAILURE
        }
    }
}

//! Routen: `api` ist die REST-API unter `/api/v1` (auch die Oberfläche nutzt nur sie),
//! `spa` liefert die Oberfläche (Dioxus/WebAssembly), `files` Dateien und Freigaben,
//! `assets` PDF.js, Schrift und Icons, `catchers` beantwortet Fehler als JSON oder HTML.

pub mod api;
pub mod assets;
pub mod catchers;
pub mod files;
pub mod spa;
pub mod upload;

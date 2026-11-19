//! Eigene Skripte und Stile der Seiten, die der Server ausliefert (PDF-Viewer, Vorschau,
//! Handy-Web-App). Tailwind liest die Klassen daraus mit (siehe `@source` der Oberfläche).

pub mod manifest;
pub mod mobile;
pub mod preview;
pub mod service_worker;
pub mod viewer;
pub mod viewer_css;

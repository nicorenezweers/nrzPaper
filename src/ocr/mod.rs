//! Texterkennung. `engine` ruft Poppler und Tesseract auf – ausschliesslich über Pipes,
//! nie mit temporären Dateien, damit kein Klartext auf der Platte landet. `queue` arbeitet
//! die Dokumente mit Status `pending` im Hintergrund ab.

pub mod engine;
pub mod queue;

pub use engine::Engine;
pub use queue::OcrQueue;

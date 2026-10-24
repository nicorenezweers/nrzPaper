//! Netzwerk-Scanner über eSCL (AirScan): Status, eine Seite vom Glas scannen, Scanner im Netz
//! suchen – und ein PDF aus den gescannten JPEG-Seiten bauen. Übernommen aus nrzDocsScanner.

mod escl;
mod pdf;

pub use escl::{discover, scan_page, status, Found};
pub use pdf::{is_jpeg, make_pdf};

use std::sync::LazyLock;
use std::time::Duration;

/// Eigener HTTP-Client für Scanner im LAN (kurze Verbindungszeit, keine Weiterleitungen).
pub(crate) static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP-Client")
});

/// Ein physischer Scanner – Aufträge laufen nacheinander.
pub static SCAN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

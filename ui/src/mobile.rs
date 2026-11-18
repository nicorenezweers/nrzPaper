//! Handy & Web-App: Service Worker (Teilen-Ziel auf Android), geteilte Dateien abholen und
//! Fotos von der Kamera hochladen. Die eigentliche Arbeit macht `static/mobile.js`
//! (Canvas, Cache-API) – hier nur die Anbindung über `document::eval`.

use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// `import()` von mobile.js, dann `call` ausführen und das Ergebnis zurückgeben.
async fn run<T: DeserializeOwned>(call: &str) -> Result<T, String> {
    let js = format!(
        "const m = await import('/static/mobile.js?v={}'); return await m.{call};",
        env!("CARGO_PKG_VERSION")
    );
    document::eval(&js).join::<T>().await.map_err(|e| format!("{e:?}"))
}

fn arg(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// Service Worker anmelden (einmal beim Start).
pub fn register_service_worker() {
    spawn(async move {
        let _ = run::<serde_json::Value>("registerServiceWorker()").await;
    });
}

#[derive(Deserialize, Default)]
pub struct Shared {
    #[serde(default)]
    pub count: usize,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub failed: bool,
}

/// Über „Teilen → nrzPaper“ (Android) gekommene Dateien in die Inbox der Organisation legen.
pub async fn take_shared(org: &str) -> Shared {
    run::<Shared>(&format!("takeShared({})", arg(org))).await.unwrap_or_default()
}

/// Fotos aus `<input id=…>` gedreht und verkleinert an `url` senden. `Ok(None)` = nichts gewählt.
pub async fn upload_photos<T: DeserializeOwned>(input_id: &str, url: &str) -> Result<Option<T>, String> {
    let v = run::<serde_json::Value>(&format!("uploadPhotos({}, {})", arg(input_id), arg(&format!("/api/v1{url}")))).await?;
    if v["empty"].as_bool() == Some(true) {
        return Ok(None);
    }
    if let Some(e) = v["error"].as_str() {
        return Err(crate::i18n::tr(e));
    }
    serde_json::from_value(v).map(Some).map_err(|e| e.to_string())
}

/// Datei für das Teilen-Menü vorladen; `true`, wenn das Gerät Dateien teilen kann.
pub async fn prepare_share(url: &str, name: &str, mime: &str) -> bool {
    run::<bool>(&format!("prepareShare({}, {}, {})", arg(url), arg(name), arg(mime))).await.unwrap_or(false)
}

/// Teilen-Menü öffnen (direkt im Klick aufrufen). `Err` nur bei echten Fehlern.
pub async fn share_file(title: &str) -> Result<(), String> {
    match run::<String>(&format!("shareFile({})", arg(title))).await?.as_str() {
        "ok" | "abgebrochen" => Ok(()),
        e => Err(e.to_string()),
    }
}

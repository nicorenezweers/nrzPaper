//! Kleine Helfer: Datum, Grösse, Browser-Speicher.

/// Unix-Sekunden → "24.10.2026, 14:05" in der Zeitzone des Browsers.
pub fn datetime(ts: i64) -> String {
    let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ts as f64 * 1000.0));
    let opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&opts, &"dateStyle".into(), &"medium".into());
    let _ = js_sys::Reflect::set(&opts, &"timeStyle".into(), &"short".into());
    d.to_locale_string("de-CH", &opts).into()
}

/// Unix-Sekunden → "24.10.2026".
pub fn date(ts: i64) -> String {
    let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ts as f64 * 1000.0));
    let opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&opts, &"dateStyle".into(), &"medium".into());
    d.to_locale_date_string("de-CH", &opts).into()
}

pub fn now() -> i64 {
    (js_sys::Date::now() / 1000.0) as i64
}

/// 812 B, 14,2 KB, 3,1 MB
pub fn size(bytes: i64) -> String {
    let b = bytes as f64;
    if b >= 1_048_576.0 {
        format!("{:.1} MB", b / 1_048_576.0).replace('.', ",")
    } else if b >= 1024.0 {
        format!("{:.1} KB", b / 1024.0).replace('.', ",")
    } else {
        format!("{bytes} B")
    }
}

pub fn storage_get(key: &str) -> Option<String> {
    web_sys::window()?.local_storage().ok()??.get_item(key).ok()?
}

pub fn storage_set(key: &str, value: &str) {
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.set_item(key, value);
    }
}

/// Text in die Zwischenablage.
pub async fn copy(text: &str) -> bool {
    let Some(w) = web_sys::window() else { return false };
    wasm_bindgen_futures::JsFuture::from(w.navigator().clipboard().write_text(text)).await.is_ok()
}

/// Ganze Adresse zu einem Pfad (für Freigabelinks).
pub fn absolute(path: &str) -> String {
    let origin = web_sys::window().and_then(|w| w.location().origin().ok()).unwrap_or_default();
    format!("{origin}{path}")
}

/// Dateien aus einem <input type=file> oder Drop-Ereignis.
pub fn files_of(list: Option<web_sys::FileList>) -> Vec<web_sys::File> {
    let Some(list) = list else { return Vec::new() };
    (0..list.length()).filter_map(|i| list.get(i)).collect()
}

/// Dateien aus einem Drop-Ereignis (Drag & Drop vom Desktop).
pub fn dropped_files(e: &dioxus::prelude::DragEvent) -> Vec<web_sys::File> {
    use wasm_bindgen::JsCast;
    e.data()
        .downcast::<web_sys::Event>()
        .and_then(|ev| ev.dyn_ref::<web_sys::DragEvent>())
        .and_then(|d| d.data_transfer())
        .map(|dt| files_of(dt.files()))
        .unwrap_or_default()
}

/// Dateien aus einem <input type=file> (onchange); leert das Feld danach.
pub fn input_files(e: &dioxus::prelude::FormEvent) -> Vec<web_sys::File> {
    use wasm_bindgen::JsCast;
    let input = e
        .data()
        .downcast::<web_sys::Event>()
        .and_then(|ev| ev.target())
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
    match input {
        Some(i) => {
            let files = files_of(i.files());
            i.set_value("");
            files
        }
        None => Vec::new(),
    }
}

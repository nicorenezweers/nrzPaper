//! KI-Analyse über OpenRouter (übernommen aus nrzDocsScanner): aus den Seiten (Bilder) oder
//! dem PDF werden Name, Tags und Metadaten als JSON gelesen – bevor etwas abgelegt wird.
//!
//! Standard ist das kostenlose `openrouter/free`. Dessen Router landet manchmal bei einem
//! ungeeigneten Modell; dann werden der Reihe nach kostenlose Bild-Modelle probiert.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

/// Eingebauter Schlüssel (beim Bauen als Secret gesetzt, nie im Quelltext), damit die App
/// ohne Einrichtung taggt.
pub const BUILTIN_KEY: Option<&str> = option_env!("NRZ_OPENROUTER_KEY");

static HTTP: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("HTTP-Client")
});

/// Zugang zu OpenRouter: Schlüssel, Modell, Ersatzmodelle.
#[derive(Clone)]
pub struct AiConfig {
    pub key: String,
    pub model: String,
    pub url: String,
    /// Weitere Modelle, wenn das gewünschte versagt (der Reihe nach).
    pub fallbacks: Vec<String>,
    /// KI ganz ausgeschaltet (Einstellungen)
    pub disabled: bool,
}

/// KI-Einstellungen in der Oberfläche (app_settings "ai"). Leere Felder = Vorgabe.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AiSettings {
    pub disabled: bool,
    /// Eigener OpenRouter-Schlüssel (sonst Umgebung bzw. eingebauter)
    pub key: String,
    pub model: String,
    /// Ersatzmodelle; leer = die eingebauten kostenlosen Bild-Modelle
    pub fallbacks: Vec<String>,
}

pub const SETTINGS_KEY: &str = "ai";

impl AiConfig {
    /// Nur aus der Konfiguration (Umgebung/eingebaut) – ohne die Einstellungen in der Datenbank.
    pub fn from(config: &crate::config::Config) -> Self {
        AiConfig {
            key: resolve_key(config.openrouter_key.as_deref().unwrap_or("")),
            model: if config.openrouter_model.trim().is_empty() {
                DEFAULT_MODEL.into()
            } else {
                config.openrouter_model.trim().into()
            },
            url: config.openrouter_url.trim_end_matches('/').to_string(),
            fallbacks: FALLBACK_MODELS.iter().map(|m| m.to_string()).collect(),
            disabled: false,
        }
    }

    /// Konfiguration plus Einstellungen aus der Oberfläche (die haben Vorrang).
    pub async fn load(pool: &sqlx::SqlitePool, config: &crate::config::Config) -> Self {
        let mut ai = Self::from(config);
        if let Ok(Some(s)) = crate::services::settings::get::<AiSettings>(pool, SETTINGS_KEY).await
        {
            if !s.key.trim().is_empty() {
                ai.key = s.key.trim().to_string();
            }
            if !s.model.trim().is_empty() {
                ai.model = s.model.trim().to_string();
            }
            if !s.fallbacks.is_empty() {
                ai.fallbacks = s.fallbacks;
            }
            ai.disabled = s.disabled;
        }
        ai
    }

    pub fn available(&self) -> bool {
        !self.disabled && !self.key.is_empty()
    }

    /// Woher der Schlüssel kommt (für die Einstellungen; der Schlüssel selbst bleibt geheim).
    pub fn key_hint(&self) -> String {
        let k: Vec<char> = self.key.chars().collect();
        if k.len() < 16 {
            return String::new();
        }
        format!(
            "{}…{}",
            k[..10].iter().collect::<String>(),
            k[k.len() - 4..].iter().collect::<String>()
        )
    }
}

pub const DEFAULT_MODEL: &str = "openrouter/free";
pub const FALLBACK_MODELS: &[&str] = &[
    "nvidia/nemotron-3.5-super-vl:free",
    "google/gemma-4-31b-it:free",
    "google/gemma-4-26b-a4b-it:free",
];

/// Felder, die aus dem Brief gelesen werden (Schlüssel → Bezeichnung in der Oberfläche).
pub const META_FIELDS: &[(&str, &str)] = &[
    ("absender", "Absender"),
    ("empfaenger", "Empfänger"),
    ("datum", "Datum"),
    ("betrag", "Betrag"),
    ("waehrung", "Währung"),
    ("referenz", "Referenz / Rechnungsnr."),
    ("kundennummer", "Kundennummer"),
    ("faellig", "Fällig am"),
    ("iban", "IBAN"),
];

pub struct TagInfo {
    pub name: String,
    pub description: String,
}

pub enum Input<'a> {
    /// Gescannte Seiten als JPEG
    Pages(&'a [Vec<u8>]),
    /// Importiertes PDF
    Pdf(&'a [u8]),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Analysis {
    pub title: String,
    pub tags: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub summary: String,
    pub model: String,
}

async fn chat(ai: &AiConfig, model: &str, messages: &Value) -> Result<(String, String), String> {
    let resp = HTTP
        .post(format!("{}/chat/completions", ai.url))
        .bearer_auth(&ai.key)
        .header("X-Title", "nrzPaper")
        .timeout(Duration::from_secs(180))
        .json(&json!({
            "model": model,
            "max_tokens": 4000,
            "temperature": 0,
            "reasoning": { "effort": "low" },
            "messages": messages,
        }))
        .send()
        .await
        .map_err(|e| format!("OpenRouter nicht erreichbar: {e}"))?;
    let status = resp.status();
    let value: Value = resp.json().await.unwrap_or_default();
    if let Some(msg) = value["error"]["message"].as_str() {
        return Err(format!("{model}: {msg} ({})", status.as_u16()));
    }
    let content = value["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if content.trim().is_empty() {
        return Err(format!("{model}: leere Antwort"));
    }
    Ok((
        content,
        value["model"].as_str().unwrap_or(model).to_string(),
    ))
}

/// Holt das JSON-Objekt aus der Antwort, auch wenn es in ```json steckt.
pub(crate) fn extract_json(text: &str) -> Option<Value> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    serde_json::from_str(&text[start..=end]).ok()
}

fn resolve_key(user_key: &str) -> String {
    if !user_key.trim().is_empty() {
        return user_key.trim().to_string();
    }
    BUILTIN_KEY.map(String::from).unwrap_or_default()
}

pub async fn analyze(
    ai: &AiConfig,
    area: &str,
    filename: &str,
    input: Input<'_>,
    tags: &[TagInfo],
) -> Result<Analysis, String> {
    if ai.disabled {
        return Err("Die KI ist in den Einstellungen ausgeschaltet.".into());
    }
    if !ai.available() {
        return Err(
            "Kein OpenRouter-Schlüssel vorhanden – unter Einstellungen → KI eintragen.".into(),
        );
    }
    let tag_list = if tags.is_empty() {
        "(keine – gib eine leere Liste zurück)".to_string()
    } else {
        tags.iter()
            .map(|t| {
                if t.description.is_empty() {
                    format!("- {}", t.name)
                } else {
                    format!("- {}: {}", t.name, t.description)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let fields = META_FIELDS
        .iter()
        .map(|(k, label)| format!("\"{k}\": \"\" ({label})"))
        .collect::<Vec<_>>()
        .join(", ");
    let system = format!(
        "Du analysierst ein gescanntes Dokument ({area}) und liest alles direkt aus dem Inhalt.\n\
         1. title: kurzer, sprechender Dateiname auf Deutsch, Muster \"JJJJ-MM-TT Absender Dokumentart Thema\", ohne Dateiendung.\n\
         2. tags: alle passenden Tags, ausschliesslich aus dieser Liste (Name: wann er passt):\n{tag_list}\n\
         3. metadata: nur was im Dokument steht, sonst leer lassen. Datumsangaben als JJJJ-MM-TT, Beträge als Zahl mit Punkt. Felder: {fields}\n\
         4. summary: ein Satz, worum es geht.\n\
         Antworte ausschliesslich mit JSON: {{\"title\": \"\", \"tags\": [], \"metadata\": {{}}, \"summary\": \"\"}}"
    );
    let mut content = vec![json!({ "type": "text", "text": format!("Dateiname: {filename}") })];
    match input {
        Input::Pages(pages) => {
            for page in pages.iter().take(4) {
                content.push(json!({ "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{}", B64.encode(page)) } }));
            }
        }
        Input::Pdf(pdf) => content.push(json!({ "type": "file", "file": {
            "filename": filename,
            "file_data": format!("data:application/pdf;base64,{}", B64.encode(pdf)),
        }})),
    }
    let messages =
        json!([{ "role": "system", "content": system }, { "role": "user", "content": content }]);

    // Gewünschtes Modell zuerst, dann die kostenlosen Bild-Modelle; das freie Routing zweimal versuchen.
    let model = ai.model.as_str();
    let mut models = vec![model.to_string()];
    if model == DEFAULT_MODEL {
        models.push(DEFAULT_MODEL.into());
    }
    models.extend(ai.fallbacks.iter().filter(|m| m.as_str() != model).cloned());
    let mut errors = Vec::new();
    for m in &models {
        match chat(ai, m, &messages).await {
            Ok((text, used)) => match extract_json(&text)
                .filter(|v| v["title"].as_str().is_some_and(|t| !t.trim().is_empty()))
            {
                Some(v) => return Ok(parse(&v, tags, used)),
                None => errors.push(format!("{used}: unbrauchbare Antwort")),
            },
            Err(e) => errors.push(e),
        }
    }
    Err(format!("KI-Analyse fehlgeschlagen – {}", errors.join("; ")))
}

pub(crate) fn parse(v: &Value, tags: &[TagInfo], model: String) -> Analysis {
    let title: String = v["title"]
        .as_str()
        .unwrap_or_default()
        .chars()
        .map(|c| {
            if "/\\:*?\"<>|\n\r".contains(c) {
                '-'
            } else {
                c
            }
        })
        .collect::<String>()
        .trim()
        .trim_end_matches(".pdf")
        .chars()
        .take(120)
        .collect();
    let chosen = v["tags"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| t.as_str())
        .filter_map(|wanted| {
            tags.iter()
                .find(|t| t.name.eq_ignore_ascii_case(wanted.trim()))
                .map(|t| t.name.clone())
        })
        .fold(Vec::<String>::new(), |mut acc, t| {
            if !acc.contains(&t) {
                acc.push(t);
            }
            acc
        });
    let metadata = META_FIELDS
        .iter()
        .filter_map(|(k, _)| {
            let raw = &v["metadata"][*k];
            let s = raw
                .as_str()
                .map(String::from)
                .or_else(|| raw.as_f64().map(|n| n.to_string()))?;
            let s = s.trim().to_string();
            (!s.is_empty()).then(|| (k.to_string(), s))
        })
        .collect();
    Analysis {
        title,
        tags: chosen,
        metadata,
        summary: v["summary"].as_str().unwrap_or_default().trim().to_string(),
        model,
    }
}

/// Kostenlose Modelle von OpenRouter, die Bilder lesen können (für die Auswahl in den Einstellungen).
pub async fn free_vision_models(ai: &AiConfig) -> Result<Vec<(String, String)>, String> {
    let resp = HTTP
        .get(format!("{}/models", ai.url))
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("OpenRouter nicht erreichbar: {e}"))?;
    let v: Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut out: Vec<(String, String)> = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| {
            m["id"]
                .as_str()
                .is_some_and(|id| id.ends_with(":free") || id == DEFAULT_MODEL)
        })
        .filter(|m| {
            m["architecture"]["input_modalities"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == "image"))
                || m["id"] == DEFAULT_MODEL
        })
        .map(|m| {
            (
                m["id"].as_str().unwrap_or_default().to_string(),
                m["name"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_model_answer() {
        let v = extract_json("```json\n{\"title\":\"2026-10-15 Swisscom Rechnung\",\"tags\":[\"rechnung\",\"Neu\"],\"metadata\":{\"betrag\":79.9,\"absender\":\"Swisscom\",\"iban\":\"\"},\"summary\":\"Rechnung\"}\n```").unwrap();
        let a = parse(
            &v,
            &[TagInfo {
                name: "Rechnung".into(),
                description: String::new(),
            }],
            "m".into(),
        );
        assert_eq!(a.title, "2026-10-15 Swisscom Rechnung");
        assert_eq!(a.tags, vec!["Rechnung"]);
        assert_eq!(a.metadata.get("betrag").map(String::as_str), Some("79.9"));
        assert!(!a.metadata.contains_key("iban"));
    }
}

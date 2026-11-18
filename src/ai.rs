//! KI-Analyse (übernommen aus nrzDocsScanner): aus den Seiten (Bilder) oder dem PDF werden
//! Name, Tags und Metadaten als JSON gelesen – bevor etwas abgelegt wird.
//!
//! Anbieter: alles, was die OpenAI-Chat-API spricht (OpenRouter, OpenAI, Google Gemini,
//! Mistral, Groq, xAI, DeepSeek, Together, Fireworks, Nvidia, Hugging Face, Azure OpenAI,
//! Ollama, LM Studio, vLLM, LocalAI und jede eigene Adresse), dazu Anthropic über die
//! Messages-API. Liest ein Modell keine Bilder, bekommt es den per Texterkennung gelesenen
//! Text; nimmt ein Anbieter keine PDFs an, gehen die ersten Seiten als Bilder.
//!
//! Standard ist das kostenlose `openrouter/free` mit eingebautem Schlüssel. Dessen Router
//! landet manchmal bei einem ungeeigneten Modell; dann werden der Reihe nach kostenlose
//! Bild-Modelle probiert.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

/// Eingebauter Schlüssel für OpenRouter (beim Bauen als Secret gesetzt, nie im Quelltext),
/// damit die App ohne Einrichtung taggt. Ein leerer Wert zählt als „keiner“: BuildKit setzt
/// die Variable bei fehlendem Secret auf "".
pub const BUILTIN_KEY: Option<&str> = match option_env!("NRZ_OPENROUTER_KEY") {
    Some(k) if !k.is_empty() => Some(k),
    _ => None,
};

static HTTP: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("HTTP-Client")
});

// ------------------------------------------------------------------ Anbieter

/// Wie mit dem Anbieter gesprochen wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// `POST {url}/chat/completions` (OpenAI-kompatibel)
    OpenAi,
    /// `POST {url}/messages` (Anthropic)
    Anthropic,
}

/// Wie der Schlüssel mitgeschickt wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Auth {
    /// `Authorization: Bearer …`
    Bearer,
    /// `api-key: …` (Azure OpenAI)
    ApiKey,
    /// `x-api-key: …` (Anthropic)
    XApiKey,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    pub protocol: Protocol,
    pub auth: Auth,
    /// Voreingestellte Adresse (leer = muss eingetragen werden)
    pub url: &'static str,
    /// Ohne Schlüssel nutzbar (lokale Server)
    pub key_optional: bool,
    /// Voreingestelltes Modell (leer = in den Einstellungen wählen)
    pub model: &'static str,
    /// Nimmt PDFs direkt an (sonst gehen die ersten Seiten als Bilder)
    pub pdf: bool,
    /// Wo es Schlüssel gibt
    pub keys_url: &'static str,
}

const fn p(
    id: &'static str,
    name: &'static str,
    url: &'static str,
    model: &'static str,
    pdf: bool,
    keys_url: &'static str,
) -> Provider {
    Provider {
        id,
        name,
        protocol: Protocol::OpenAi,
        auth: Auth::Bearer,
        url,
        key_optional: false,
        model,
        pdf,
        keys_url,
    }
}

const fn local(id: &'static str, name: &'static str, url: &'static str, model: &'static str) -> Provider {
    Provider {
        key_optional: true,
        ..p(id, name, url, model, false, "")
    }
}

pub const PROVIDERS: &[Provider] = &[
    p("openrouter", "OpenRouter", "https://openrouter.ai/api/v1", DEFAULT_MODEL, true, "https://openrouter.ai/keys"),
    p("openai", "OpenAI", "https://api.openai.com/v1", "gpt-4o-mini", true, "https://platform.openai.com/api-keys"),
    Provider {
        protocol: Protocol::Anthropic,
        auth: Auth::XApiKey,
        ..p("anthropic", "Anthropic (Claude)", "https://api.anthropic.com/v1", "claude-haiku-5-5", true, "https://console.anthropic.com/settings/keys")
    },
    p("gemini", "Google Gemini", "https://generativelanguage.googleapis.com/v1beta/openai", "gemini-2.5-flash", false, "https://aistudio.google.com/apikey"),
    p("mistral", "Mistral AI", "https://api.mistral.ai/v1", "mistral-small-latest", false, "https://console.mistral.ai/api-keys"),
    p("xai", "xAI (Grok)", "https://api.x.ai/v1", "", false, "https://console.x.ai"),
    p("groq", "Groq", "https://api.groq.com/openai/v1", "meta-llama/llama-4-scout-17b-16e-instruct", false, "https://console.groq.com/keys"),
    p("deepseek", "DeepSeek", "https://api.deepseek.com/v1", "deepseek-chat", false, "https://platform.deepseek.com/api_keys"),
    p("together", "Together AI", "https://api.together.xyz/v1", "meta-llama/Llama-4-Scout-17B-16E-Instruct", false, "https://api.together.ai/settings/api-keys"),
    p("fireworks", "Fireworks AI", "https://api.fireworks.ai/inference/v1", "", false, "https://fireworks.ai/account/api-keys"),
    p("perplexity", "Perplexity", "https://api.perplexity.ai", "sonar", false, "https://www.perplexity.ai/settings/api"),
    p("cohere", "Cohere", "https://api.cohere.ai/compatibility/v1", "command-a-vision-07-2025", false, "https://dashboard.cohere.com/api-keys"),
    p("nvidia", "NVIDIA NIM", "https://integrate.api.nvidia.com/v1", "", false, "https://build.nvidia.com"),
    p("huggingface", "Hugging Face", "https://router.huggingface.co/v1", "", false, "https://huggingface.co/settings/tokens"),
    p("qwen", "Alibaba Qwen (DashScope)", "https://dashscope-intl.aliyuncs.com/compatible-mode/v1", "qwen-vl-max", false, "https://dashscope.console.aliyun.com/apiKey"),
    Provider {
        auth: Auth::ApiKey,
        ..p("azure", "Azure OpenAI", "", "", true, "https://portal.azure.com")
    },
    local("ollama", "Ollama (lokal)", "http://localhost:11434/v1", "llama3.2-vision"),
    local("lmstudio", "LM Studio (lokal)", "http://localhost:1234/v1", ""),
    local("vllm", "vLLM / LocalAI (lokal)", "http://localhost:8000/v1", ""),
    Provider {
        key_optional: true,
        ..p("custom", "Eigene OpenAI-kompatible API", "", "", false, "")
    },
];

pub fn provider(id: &str) -> &'static Provider {
    PROVIDERS.iter().find(|p| p.id == id).unwrap_or(&PROVIDERS[0])
}

// ------------------------------------------------------------------ Einstellungen

/// Zugang zur KI: Anbieter, Schlüssel, Adresse, Modell, Ersatzmodelle.
#[derive(Clone)]
pub struct AiConfig {
    pub provider: &'static Provider,
    pub key: String,
    pub model: String,
    pub url: String,
    /// Weitere Modelle, wenn das gewünschte versagt (der Reihe nach).
    pub fallbacks: Vec<String>,
    /// KI ganz ausgeschaltet (Einstellungen)
    pub disabled: bool,
    /// Modell liest keine Bilder: stattdessen den erkannten Text schicken
    pub text_only: bool,
    /// Sprachen der Texterkennung (für `text_only`)
    pub ocr_languages: String,
    /// Sprache von Titel und Zusammenfassung: Code (de, en, …), "auto" = wie das Dokument
    pub language: String,
}

/// Sprachen für Titel und Zusammenfassung der KI.
pub const SUGGESTION_LANGUAGES: &[(&str, &str)] = &[
    ("de", "Deutsch"),
    ("en", "English"),
    ("fr", "français"),
    ("it", "italiano"),
    ("es", "español"),
    ("pt", "português"),
    ("nl", "Nederlands"),
];

fn language_instruction(code: &str) -> String {
    match SUGGESTION_LANGUAGES.iter().find(|(c, _)| *c == code) {
        Some((_, name)) => format!("auf {name}"),
        None if code == "auto" => "in der Sprache des Dokuments".into(),
        None => "auf Deutsch".into(),
    }
}

/// KI-Einstellungen in der Oberfläche (app_settings "ai"). Leere Felder = Vorgabe.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AiSettings {
    pub disabled: bool,
    /// Gewählter Anbieter (leer = OpenRouter)
    pub provider: String,
    /// Eigener OpenRouter-Schlüssel aus 1.3 (gilt weiter für OpenRouter)
    pub key: String,
    /// Schlüssel je Anbieter – beim Wechseln geht keiner verloren
    pub keys: BTreeMap<String, String>,
    /// Eigene Adresse je Anbieter (leer = Vorgabe des Anbieters)
    pub urls: BTreeMap<String, String>,
    pub model: String,
    /// Ersatzmodelle; leer = bei OpenRouter die eingebauten kostenlosen Bild-Modelle
    pub fallbacks: Vec<String>,
    pub text_only: bool,
    /// Sprache der Vorschläge (leer = Deutsch, "auto" = wie das Dokument)
    pub language: String,
}

impl AiSettings {
    pub fn provider_id(&self) -> &str {
        if self.provider.is_empty() {
            "openrouter"
        } else {
            &self.provider
        }
    }

    /// Hinterlegter Schlüssel für einen Anbieter.
    pub fn key_for(&self, provider: &str) -> &str {
        match self.keys.get(provider).map(|k| k.trim()).filter(|k| !k.is_empty()) {
            Some(k) => k,
            None if provider == "openrouter" => self.key.trim(),
            None => "",
        }
    }
}

pub const SETTINGS_KEY: &str = "ai";

/// Woher der Schlüssel kommt (für die Einstellungen; der Schlüssel selbst bleibt geheim).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeySource {
    Settings,
    Environment,
    Builtin,
    None,
}

impl AiConfig {
    /// Nur aus der Konfiguration (Umgebung/eingebaut) – ohne die Einstellungen in der Datenbank.
    pub fn from(config: &crate::config::Config) -> Self {
        let provider = provider(config.ai_provider.trim());
        let openrouter = provider.id == "openrouter";
        let env_key = match (&config.ai_key, &config.openrouter_key) {
            (Some(k), _) if !k.trim().is_empty() => k.trim().to_string(),
            (_, Some(k)) if openrouter && !k.trim().is_empty() => k.trim().to_string(),
            _ => String::new(),
        };
        let key = if env_key.is_empty() && openrouter {
            BUILTIN_KEY.map(String::from).unwrap_or_default()
        } else {
            env_key
        };
        let url = if !config.ai_url.trim().is_empty() {
            config.ai_url.trim()
        } else if openrouter {
            config.openrouter_url.trim()
        } else {
            provider.url
        };
        let model = if !config.ai_model.trim().is_empty() {
            config.ai_model.trim().to_string()
        } else if openrouter && !config.openrouter_model.trim().is_empty() {
            config.openrouter_model.trim().to_string()
        } else {
            provider.model.to_string()
        };
        AiConfig {
            provider,
            key,
            model,
            url: url.trim_end_matches('/').to_string(),
            fallbacks: if openrouter {
                FALLBACK_MODELS.iter().map(|m| m.to_string()).collect()
            } else {
                Vec::new()
            },
            disabled: false,
            text_only: false,
            ocr_languages: config.ocr_languages.clone(),
            language: "de".into(),
        }
    }

    /// Konfiguration plus Einstellungen aus der Oberfläche (die haben Vorrang).
    pub async fn load(pool: &sqlx::SqlitePool, config: &crate::config::Config) -> Self {
        let stored = crate::services::settings::get::<AiSettings>(pool, SETTINGS_KEY)
            .await
            .ok()
            .flatten();
        Self::with_settings(config, stored.as_ref())
    }

    pub fn with_settings(config: &crate::config::Config, s: Option<&AiSettings>) -> Self {
        let mut ai = Self::from(config);
        let Some(s) = s else { return ai };
        let base = ai.provider.id;
        let chosen = provider(s.provider_id());
        if chosen.id != base {
            // Anderer Anbieter als in der Umgebung: dessen Vorgaben, kein Umgebungs-Schlüssel
            ai.provider = chosen;
            ai.key = String::new();
            ai.url = chosen.url.to_string();
            ai.model = chosen.model.to_string();
            ai.fallbacks = Vec::new();
            if chosen.id == "openrouter" {
                ai.key = BUILTIN_KEY.map(String::from).unwrap_or_default();
                ai.url = config.openrouter_url.trim_end_matches('/').to_string();
                ai.fallbacks = FALLBACK_MODELS.iter().map(|m| m.to_string()).collect();
            }
        }
        let key = s.key_for(chosen.id);
        if !key.is_empty() {
            ai.key = key.to_string();
        }
        if let Some(url) = s.urls.get(chosen.id).map(|u| u.trim()).filter(|u| !u.is_empty()) {
            ai.url = url.trim_end_matches('/').to_string();
        }
        if !s.model.trim().is_empty() {
            ai.model = s.model.trim().to_string();
        }
        if !s.fallbacks.is_empty() {
            ai.fallbacks = s.fallbacks.clone();
        }
        ai.disabled = s.disabled;
        ai.text_only = s.text_only;
        if !s.language.is_empty() {
            ai.language = s.language.clone();
        }
        ai
    }

    /// Woher der wirksame Schlüssel kommt.
    pub fn key_source(config: &crate::config::Config, s: Option<&AiSettings>) -> KeySource {
        let id = s.map(|s| s.provider_id()).unwrap_or(config.ai_provider.trim());
        let id = provider(id).id;
        if s.is_some_and(|s| !s.key_for(id).is_empty()) {
            return KeySource::Settings;
        }
        let env_provider = provider(config.ai_provider.trim()).id;
        if env_provider == id
            && (config.ai_key.as_deref().is_some_and(|k| !k.trim().is_empty())
                || (id == "openrouter" && config.openrouter_key.as_deref().is_some_and(|k| !k.trim().is_empty())))
        {
            return KeySource::Environment;
        }
        if id == "openrouter" && BUILTIN_KEY.is_some() {
            return KeySource::Builtin;
        }
        KeySource::None
    }

    pub fn available(&self) -> bool {
        !self.disabled
            && (!self.key.is_empty() || self.provider.key_optional)
            && !self.url.is_empty()
            && !self.model.is_empty()
    }

    /// Kurzform des Schlüssels für die Anzeige (Anfang und Ende).
    pub fn key_hint(&self) -> String {
        hint(&self.key)
    }

    /// Warum die KI nicht bereit ist (für Meldungen).
    pub fn missing(&self) -> Option<String> {
        if self.disabled {
            Some("Die KI ist in den Einstellungen ausgeschaltet.".into())
        } else if self.key.is_empty() && !self.provider.key_optional {
            Some(format!(
                "Kein Schlüssel für {} – unter Einstellungen → KI eintragen.",
                self.provider.name
            ))
        } else if self.url.is_empty() {
            Some(format!("Für {} fehlt die Adresse der API.", self.provider.name))
        } else if self.model.is_empty() {
            Some(format!("Für {} ist kein Modell gewählt.", self.provider.name))
        } else {
            None
        }
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let mut req = HTTP.request(method, format!("{}/{path}", self.url));
        if !self.key.is_empty() {
            req = match self.provider.auth {
                Auth::Bearer => req.bearer_auth(&self.key),
                Auth::ApiKey => req.header("api-key", &self.key),
                Auth::XApiKey => req.header("x-api-key", &self.key),
            };
        }
        if self.provider.protocol == Protocol::Anthropic {
            req = req.header("anthropic-version", "2023-06-01");
        }
        if self.provider.id == "openrouter" {
            req = req.header("X-Title", "nrzPaper");
        }
        req
    }
}

pub fn hint(key: &str) -> String {
    let k: Vec<char> = key.chars().collect();
    if k.len() < 16 {
        return String::new();
    }
    format!(
        "{}…{}",
        k[..10].iter().collect::<String>(),
        k[k.len() - 4..].iter().collect::<String>()
    )
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

// ------------------------------------------------------------------ Anfragen

/// Was an die KI geht: Text plus Bilder oder ein PDF.
enum Part {
    Text(String),
    Jpeg(Vec<u8>),
    Pdf(String, Vec<u8>),
}

fn openai_body(model: &str, system: &str, parts: &[Part], openrouter: bool) -> Value {
    let content: Vec<Value> = parts
        .iter()
        .map(|p| match p {
            Part::Text(t) => json!({ "type": "text", "text": t }),
            Part::Jpeg(j) => json!({ "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{}", B64.encode(j)) } }),
            Part::Pdf(name, d) => json!({ "type": "file", "file": {
                "filename": name,
                "file_data": format!("data:application/pdf;base64,{}", B64.encode(d)),
            }}),
        })
        .collect();
    let mut body = json!({
        "model": model,
        "max_tokens": 4000,
        "temperature": 0,
        "messages": [{ "role": "system", "content": system }, { "role": "user", "content": content }],
    });
    if openrouter {
        body["reasoning"] = json!({ "effort": "low" });
    }
    body
}

fn anthropic_body(model: &str, system: &str, parts: &[Part]) -> Value {
    let content: Vec<Value> = parts
        .iter()
        .map(|p| match p {
            Part::Text(t) => json!({ "type": "text", "text": t }),
            Part::Jpeg(j) => json!({ "type": "image", "source": { "type": "base64", "media_type": "image/jpeg", "data": B64.encode(j) } }),
            Part::Pdf(_, d) => json!({ "type": "document", "source": { "type": "base64", "media_type": "application/pdf", "data": B64.encode(d) } }),
        })
        .collect();
    json!({
        "model": model,
        "max_tokens": 4000,
        "temperature": 0,
        "system": system,
        "messages": [{ "role": "user", "content": content }],
    })
}

async fn chat(ai: &AiConfig, model: &str, system: &str, parts: &[Part]) -> Result<(String, String), String> {
    let anthropic = ai.provider.protocol == Protocol::Anthropic;
    let (path, body) = if anthropic {
        ("messages", anthropic_body(model, system, parts))
    } else {
        ("chat/completions", openai_body(model, system, parts, ai.provider.id == "openrouter"))
    };
    let resp = ai
        .request(reqwest::Method::POST, path)
        .timeout(Duration::from_secs(180))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("{} nicht erreichbar: {e}", ai.provider.name))?;
    let status = resp.status();
    let value: Value = resp.json().await.unwrap_or_default();
    let error = value["error"]["message"]
        .as_str()
        .or_else(|| value["error"].as_str())
        .or_else(|| value["message"].as_str().filter(|_| !status.is_success()));
    if let Some(msg) = error {
        return Err(format!("{model}: {msg} ({})", status.as_u16()));
    }
    if !status.is_success() {
        return Err(format!("{model}: HTTP {}", status.as_u16()));
    }
    let content = if anthropic {
        value["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|c| c["text"].as_str())
            .collect::<Vec<_>>()
            .join("")
    } else {
        value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };
    if content.trim().is_empty() {
        return Err(format!("{model}: leere Antwort"));
    }
    Ok((content, value["model"].as_str().unwrap_or(model).to_string()))
}

/// Erste Seiten eines PDFs als JPEG (für Anbieter ohne PDF-Eingang und zum Drucken).
pub async fn pdf_pages(pdf: &[u8], max: u32, dpi: u32) -> Vec<Vec<u8>> {
    let mut pages = Vec::new();
    let dpi = dpi.to_string();
    for n in 1..=max {
        let n = n.to_string();
        let args = ["-jpeg", "-jpegopt", "quality=85", "-r", &dpi, "-singlefile", "-f", &n, "-l", &n, "fd://0", "-"];
        match crate::ocr::engine::run("pdftocairo", &args, pdf).await {
            Ok(out) if !out.is_empty() => pages.push(out),
            _ => break,
        }
    }
    pages
}

/// Text per Texterkennung (für Modelle ohne Bildeingang).
async fn ocr_text(ai: &AiConfig, input: &Input<'_>) -> Result<String, String> {
    let engine = crate::ocr::Engine::detect(&ai.ocr_languages).await;
    let mut text = String::new();
    match input {
        Input::Pdf(pdf) => {
            if let Some(x) = engine.extract("application/pdf", pdf).await? {
                text = x.text;
            }
        }
        Input::Pages(pages) => {
            for page in pages.iter().take(6) {
                if let Some(x) = engine.extract("image/jpeg", page).await? {
                    text.push_str(&x.text);
                    text.push('\n');
                }
            }
        }
    }
    if text.trim().is_empty() {
        return Err("Die Texterkennung fand keinen Text – für Bilder ein Modell mit Bildeingang wählen.".into());
    }
    Ok(text.chars().take(24_000).collect())
}

pub async fn analyze(
    ai: &AiConfig,
    area: &str,
    filename: &str,
    input: Input<'_>,
    tags: &[TagInfo],
) -> Result<Analysis, String> {
    if let Some(why) = ai.missing() {
        return Err(why);
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
         1. title: kurzer, sprechender Dateiname {lang}, Muster \"JJJJ-MM-TT Absender Dokumentart Thema\", ohne Dateiendung.\n\
         2. tags: alle passenden Tags, ausschliesslich aus dieser Liste (Name: wann er passt):\n{tag_list}\n\
         3. metadata: nur was im Dokument steht, sonst leer lassen. Datumsangaben als JJJJ-MM-TT, Beträge als Zahl mit Punkt. Felder: {fields}\n\
         4. summary: ein Satz {lang}, worum es geht.\n\
         Antworte ausschliesslich mit JSON: {{\"title\": \"\", \"tags\": [], \"metadata\": {{}}, \"summary\": \"\"}}",
        lang = language_instruction(&ai.language),
    );
    let mut parts = vec![Part::Text(format!("Dateiname: {filename}"))];
    if ai.text_only {
        let text = ocr_text(ai, &input).await?;
        parts.push(Part::Text(format!("Inhalt (per Texterkennung gelesen):\n{text}")));
    } else {
        match input {
            Input::Pages(pages) => parts.extend(pages.iter().take(4).map(|p| Part::Jpeg(p.clone()))),
            Input::Pdf(pdf) if ai.provider.pdf => parts.push(Part::Pdf(filename.to_string(), pdf.to_vec())),
            Input::Pdf(pdf) => {
                let pages = pdf_pages(pdf, 4, 110).await;
                if pages.is_empty() {
                    return Err("Das PDF liess sich nicht in Bilder umwandeln (Poppler fehlt?).".into());
                }
                parts.extend(pages.into_iter().map(Part::Jpeg));
            }
        }
    }

    // Gewünschtes Modell zuerst, dann die Ersatzmodelle; das freie Routing zweimal versuchen.
    let model = ai.model.as_str();
    let mut models = vec![model.to_string()];
    if model == DEFAULT_MODEL && ai.provider.id == "openrouter" {
        models.push(DEFAULT_MODEL.into());
    }
    models.extend(ai.fallbacks.iter().filter(|m| m.as_str() != model).cloned());
    let mut errors = Vec::new();
    for m in &models {
        match chat(ai, m, &system, &parts).await {
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

/// Reine Textanfrage (Assistent): gewünschtes Modell, dann die Ersatzmodelle. Gibt Antwort
/// und tatsächlich genutztes Modell zurück.
pub async fn complete(ai: &AiConfig, system: &str, prompt: &str) -> Result<(String, String), String> {
    if let Some(why) = ai.missing() {
        return Err(why);
    }
    let parts = [Part::Text(prompt.to_string())];
    let mut models = vec![ai.model.clone()];
    if ai.model == DEFAULT_MODEL && ai.provider.id == "openrouter" {
        models.push(DEFAULT_MODEL.into());
    }
    models.extend(ai.fallbacks.iter().filter(|m| **m != ai.model).cloned());
    let mut errors = Vec::new();
    for m in &models {
        match chat(ai, m, system, &parts).await {
            Ok(r) => return Ok(r),
            Err(e) => errors.push(e),
        }
    }
    Err(format!("KI nicht erreichbar – {}", errors.join("; ")))
}

pub(crate) fn extract_json(text: &str) -> Option<Value> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    serde_json::from_str(&text[start..=end]).ok()
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

// ------------------------------------------------------------------ Modelle

/// Ein Modell aus der Liste des Anbieters.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    /// Kostenlos (nur bei OpenRouter bekannt)
    pub free: bool,
    /// Liest Bilder – `None`: der Anbieter verrät es nicht
    pub vision: Option<bool>,
}

/// Modelle des Anbieters (`GET {url}/models`). Bei OpenRouter mit Preis und Bildeingang.
pub async fn models(ai: &AiConfig) -> Result<Vec<ModelInfo>, String> {
    if ai.url.is_empty() {
        return Err(format!("Für {} fehlt die Adresse der API.", ai.provider.name));
    }
    let resp = ai
        .request(reqwest::Method::GET, "models")
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("{} nicht erreichbar: {e}", ai.provider.name))?;
    let status = resp.status();
    let v: Value = resp.json().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let msg = v["error"]["message"].as_str().or_else(|| v["error"].as_str()).unwrap_or("");
        return Err(format!("{}: HTTP {} {msg}", ai.provider.name, status.as_u16()));
    }
    let list = v["data"].as_array().or_else(|| v["models"].as_array());
    let mut out: Vec<ModelInfo> = list
        .into_iter()
        .flatten()
        .filter_map(|m| {
            let id = m["id"].as_str().or_else(|| m["name"].as_str())?;
            let id = id.trim_start_matches("models/").to_string();
            let name = m["display_name"]
                .as_str()
                .or_else(|| m["name"].as_str())
                .unwrap_or(&id)
                .trim_start_matches("models/")
                .to_string();
            let vision = m["architecture"]["input_modalities"]
                .as_array()
                .map(|a| a.iter().any(|x| x == "image"));
            let free = id.ends_with(":free")
                || id == DEFAULT_MODEL
                || (m["pricing"]["prompt"].as_str() == Some("0") && m["pricing"]["completion"].as_str() == Some("0"));
            Some(ModelInfo { id, name, free, vision })
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out.dedup_by(|a, b| a.id == b.id);
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

    #[test]
    fn provider_catalog_is_consistent() {
        let mut ids: Vec<&str> = PROVIDERS.iter().map(|p| p.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), PROVIDERS.len(), "IDs eindeutig");
        assert_eq!(provider("gibt-es-nicht").id, "openrouter");
        for p in PROVIDERS {
            assert!(p.url.is_empty() || p.url.starts_with("http"), "{}", p.id);
            assert!(!p.url.ends_with('/'), "{}", p.id);
        }
    }

    #[test]
    fn settings_choose_provider_key_and_url() {
        let config = crate::config::Config::default();
        let s = AiSettings {
            provider: "ollama".into(),
            model: "llava".into(),
            urls: [("ollama".to_string(), "http://192.168.0.5:11434/v1/".to_string())].into(),
            ..Default::default()
        };
        let ai = AiConfig::with_settings(&config, Some(&s));
        assert_eq!(ai.provider.id, "ollama");
        assert_eq!(ai.url, "http://192.168.0.5:11434/v1");
        assert!(ai.available(), "Ollama braucht keinen Schlüssel");
        assert_eq!(AiConfig::key_source(&config, Some(&s)), KeySource::None);

        let s = AiSettings {
            provider: "anthropic".into(),
            keys: [("anthropic".to_string(), "sk-ant-test-0123456789".to_string())].into(),
            ..Default::default()
        };
        let ai = AiConfig::with_settings(&config, Some(&s));
        assert_eq!((ai.url.as_str(), ai.model.as_str()), ("https://api.anthropic.com/v1", "claude-haiku-5-5"));
        assert_eq!(ai.key, "sk-ant-test-0123456789");
        assert!(ai.fallbacks.is_empty());
        assert_eq!(AiConfig::key_source(&config, Some(&s)), KeySource::Settings);

        // Schlüssel aus 1.3 gilt weiter für OpenRouter
        let s = AiSettings { key: "sk-or-alt-0123456789".into(), ..Default::default() };
        assert_eq!(AiConfig::with_settings(&config, Some(&s)).key, "sk-or-alt-0123456789");

        // Ohne Schlüssel und ohne Modell: nicht bereit, mit verständlichem Grund
        let s = AiSettings { provider: "xai".into(), ..Default::default() };
        let ai = AiConfig::with_settings(&config, Some(&s));
        assert!(!ai.available());
        assert!(ai.missing().unwrap().contains("xAI"));
    }

    #[test]
    fn anthropic_and_openai_bodies() {
        let parts = [Part::Text("t".into()), Part::Jpeg(vec![1, 2]), Part::Pdf("a.pdf".into(), vec![3])];
        let a = anthropic_body("claude", "sys", &parts);
        assert_eq!(a["system"], "sys");
        assert_eq!(a["messages"][0]["content"][1]["type"], "image");
        assert_eq!(a["messages"][0]["content"][2]["type"], "document");
        let o = openai_body("gpt", "sys", &parts, false);
        assert_eq!(o["messages"][0]["role"], "system");
        assert_eq!(o["messages"][1]["content"][1]["type"], "image_url");
        assert!(o.get("reasoning").is_none(), "nur OpenRouter kennt reasoning");
        assert!(openai_body("m", "s", &parts, true).get("reasoning").is_some());
    }
}

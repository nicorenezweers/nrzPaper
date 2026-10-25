//! Dieter, der Assistent: beantwortet Fragen zu den Dokumenten einer Organisation – mit
//! Quellen. Ablauf (ohne Vektor-Datenbank, nur mit der vorhandenen Volltextsuche):
//! 1. Die KI macht aus der Frage (und dem Verlauf) Suchbegriffe.
//! 2. Die Volltextsuche findet passende Dokumente; aus jedem werden die Textstellen um die
//!    Treffer genommen (plus Name, Datum, Tags und Eigenschaften).
//! 3. Die KI antwortet nur anhand dieser Auszüge und nennt die Dokumente, auf die sie sich
//!    stützt – die Oberfläche verlinkt sie.

use crate::ai::{self, AiConfig};
use crate::db::models::Document;
use crate::error::{AppError, AppResult};
use crate::search;
use crate::services::{documents, properties, tags};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;

/// So viele Dokumente gehen höchstens an die KI, so viel Text insgesamt.
const MAX_DOCS: usize = 40;
const BUDGET: usize = 40_000;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    /// "user" oder "assistant"
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    pub answer: String,
    pub sources: Vec<Source>,
    pub queries: Vec<String>,
    pub model: String,
    /// So viele Dokumente hat Dieter gelesen
    pub considered: usize,
}

fn transcript(messages: &[Message]) -> String {
    messages
        .iter()
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|m| {
            let who = if m.role == "assistant" { "Dieter" } else { "Nutzer" };
            format!("{who}: {}", m.content.chars().take(2000).collect::<String>())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Schritt 1: Suchbegriffe. Fällt die KI aus, gelten die Wörter der Frage.
async fn queries(ai: &AiConfig, messages: &[Message]) -> Vec<String> {
    let system = "Du hilfst, in einem Dokumentenarchiv (Briefe, Rechnungen, Verträge, Lohnabrechnungen, Versicherungen …) zu suchen. \
        Erzeuge aus der letzten Frage des Nutzers (mit dem Verlauf als Kontext) 1 bis 5 kurze Suchanfragen für eine Volltextsuche: \
        je 1–3 Stichwörter, in der Sprache der Dokumente (meist Deutsch), auch Synonyme und Firmennamen \
        (z. B. „Versicherungsnummer“, „Police“, „AHV“). Keine Füllwörter. \
        Antworte nur mit JSON: {\"queries\": [\"…\"]}";
    let fallback = || {
        let last = messages.iter().rev().find(|m| m.role == "user").map(|m| m.content.clone()).unwrap_or_default();
        vec![last
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.chars().count() >= 4)
            .take(6)
            .collect::<Vec<_>>()
            .join(" ")]
    };
    match ai::complete(ai, system, &transcript(messages)).await {
        Ok((text, _)) => {
            let qs: Vec<String> = ai::extract_json(&text)
                .and_then(|v| v["queries"].as_array().cloned())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|q| q.as_str().map(|s| s.trim().chars().take(80).collect::<String>()))
                .filter(|q| !q.is_empty())
                .take(5)
                .collect();
            if qs.is_empty() {
                fallback()
            } else {
                qs
            }
        }
        Err(_) => fallback(),
    }
}

/// Grober Wortstamm: übliche Endungen ab (Rechnungen → Rechnung, Policen → Polic), damit die
/// Präfixsuche auch andere Formen findet. Kurze Wörter bleiben, wie sie sind.
pub fn stem(word: &str) -> String {
    let w = word.to_lowercase();
    if w.chars().count() < 6 {
        return w;
    }
    for end in ["ungen", "en", "er", "es", "e", "n", "s"] {
        if let Some(s) = w.strip_suffix(end) {
            if s.chars().count() >= 4 {
                return if end == "ungen" { format!("{s}ung") } else { s.to_string() };
            }
        }
    }
    w
}

/// Wörter einer Suchanfrage (für die Textstellen und die Einzelsuche), samt Wortstämmen.
fn words(queries: &[String]) -> Vec<String> {
    let mut out: Vec<String> = queries
        .iter()
        .flat_map(|q| q.split(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.chars().count() >= 3)
        .flat_map(|w| [w.to_lowercase(), stem(w)])
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Textstellen um die Treffer (je ±`radius` Zeichen), zusammengeführt, höchstens `budget` Zeichen.
/// Ohne Treffer: der Anfang des Dokuments.
pub fn snippets(content: &str, words: &[String], budget: usize) -> String {
    let chars: Vec<char> = content.chars().collect();
    if chars.len() <= budget {
        return content.to_string();
    }
    let lower: Vec<char> = content.to_lowercase().chars().collect();
    // Kleinbuchstaben können länger werden (selten) – dann nur den Anfang nehmen
    if lower.len() != chars.len() {
        return chars[..budget].iter().collect();
    }
    let radius = 300usize;
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for w in words {
        let wc: Vec<char> = w.chars().collect();
        let mut i = 0;
        while i + wc.len() <= lower.len() {
            if lower[i..i + wc.len()] == wc[..] {
                ranges.push((i.saturating_sub(radius), (i + wc.len() + radius).min(chars.len())));
                i += wc.len().max(1) + radius;
            } else {
                i += 1;
            }
        }
    }
    if ranges.is_empty() {
        return chars[..budget].iter().collect();
    }
    ranges.sort();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for r in ranges {
        match merged.last_mut() {
            Some(last) if r.0 <= last.1 => last.1 = last.1.max(r.1),
            _ => merged.push(r),
        }
    }
    let mut out = String::new();
    for (a, b) in merged {
        if out.chars().count() >= budget {
            break;
        }
        if !out.is_empty() {
            out.push_str(" … ");
        }
        let take = (b - a).min(budget.saturating_sub(out.chars().count()));
        out.extend(chars[a..a + take].iter());
    }
    out
}

fn date(ts: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(ts)
        .map(|d| format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day()))
        .unwrap_or_default()
}

pub async fn ask(pool: &SqlitePool, ai: &AiConfig, org: &str, messages: &[Message]) -> AppResult<Answer> {
    if let Some(why) = ai.missing() {
        return Err(AppError::bad(why));
    }
    let question = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.trim().to_string())
        .unwrap_or_default();
    if question.is_empty() {
        return Err(AppError::bad("Bitte eine Frage stellen."));
    }
    if question.chars().count() > 2000 {
        return Err(AppError::bad("Die Frage ist zu lang (höchstens 2000 Zeichen)."));
    }

    // 1 + 2: suchen, Treffer der Reihe nach sammeln
    let qs = queries(ai, messages).await;
    let mut seen = HashSet::new();
    let mut found: Vec<Document> = Vec::new();
    let mut tried = qs.clone();
    // Breiter suchen: auch jedes Stichwort einzeln
    for w in words(&qs) {
        if !tried.iter().any(|q| q.eq_ignore_ascii_case(&w)) {
            tried.push(w);
        }
    }
    // Nennt die Frage einen Tag (auch in anderer Form, z. B. „Rechnungen“ → Tag „Rechnung“),
    // kommen dessen Dokumente zuerst
    let question_words: Vec<String> = words(std::slice::from_ref(&question));
    for (t, _) in tags::list(pool, org).await? {
        let ts = stem(&t.name);
        if question_words.iter().any(|w| *w == ts || stem(w) == ts) {
            let q = if t.name.contains(' ') { format!("tag:\"{}\"", t.name) } else { format!("tag:{}", t.name) };
            tried.insert(0, q);
        }
    }
    for q in &tried {
        let (docs, _) = search::search(pool, org, &search::parse(q), 0, 25).await?;
        for d in docs {
            if found.len() >= MAX_DOCS {
                break;
            }
            if seen.insert(d.id.clone()) {
                found.push(d);
            }
        }
    }

    // Auszüge: Budget je Dokument, Textstellen um die Treffer
    let ws = words(&qs);
    let per_doc = if found.is_empty() { 0 } else { (BUDGET / found.len()).clamp(500, 6000) };
    let mut context = String::new();
    let mut doc_tags = Vec::new();
    for (i, d) in found.iter().enumerate() {
        let full = documents::get(pool, org, &d.id).await?;
        let t: Vec<String> = tags::for_documents(pool, std::slice::from_ref(&d.id))
            .await?
            .remove(&d.id)
            .unwrap_or_default()
            .into_iter()
            .map(|t| t.name)
            .collect();
        let props: Vec<String> = properties::values(pool, org, &d.id)
            .await?
            .into_iter()
            .filter(|v| !v.display.is_empty())
            .map(|v| format!("{}: {}", v.name, v.display.join(", ")))
            .collect();
        context.push_str(&format!(
            "\n[D{}] {}\nEingang: {} · Tags: {}{}\n{}\n",
            i + 1,
            full.name,
            date(full.created_at),
            if t.is_empty() { "–".into() } else { t.join(", ") },
            if props.is_empty() { String::new() } else { format!("\nEigenschaften: {}", props.join("; ")) },
            snippets(&full.content, &ws, per_doc),
        ));
        doc_tags.push(t);
    }

    // 3: antworten
    let system = "Du bist Dieter, der freundliche Assistent von nrzPaper, einer Dokumentenverwaltung. \
        Du beantwortest Fragen ausschliesslich anhand der Auszüge aus den Dokumenten unten – nichts erfinden. \
        Steht die Antwort nicht darin, sag das ehrlich und schlage vor, wonach man suchen könnte. \
        Antworte in der Sprache der Frage, knapp und konkret (Nummern, Beträge, Daten genau übernehmen). \
        Werden mehrere Dokumente verlangt (z. B. alle Lohnabrechnungen eines Arbeitgebers in einem Zeitraum), \
        nenne alle passenden – achte auf Datum und Absender. \
        Antworte nur mit JSON: {\"answer\": \"Antwort als Text (Markdown erlaubt, Listen mit -)\", \"sources\": [\"D1\", \"D3\"]} \
        – in sources genau die Dokumente, auf die sich die Antwort stützt (bei Listen: alle aufgezählten). \
        Im Antworttext keine Kürzel wie D1 oder [D2] nennen – die Quellen zeigt die Oberfläche darunter.";
    let prompt = format!(
        "Heute: {}\n\nVerlauf:\n{}\n\nDokumente ({} gefunden):\n{}",
        date(crate::db::models::now()),
        transcript(messages),
        found.len(),
        if found.is_empty() { "(keine Treffer)".to_string() } else { context }
    );
    let (text, model) = ai::complete(ai, system, &prompt).await.map_err(AppError::bad)?;
    let (answer, refs) = match ai::extract_json(&text) {
        Some(v) => (
            v["answer"].as_str().unwrap_or("").trim().to_string(),
            v["sources"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| s.as_str().or_else(|| s.as_i64().map(|_| "")).map(String::from))
                .collect::<Vec<_>>(),
        ),
        // Kein JSON: Antwort als Text nehmen, Quellen aus [D…] im Text
        None => (text.trim().to_string(), Vec::new()),
    };
    // Verweise wie [D3], (D1, D2) im Text: als Quellen nehmen und aus dem Text entfernen
    static CITE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"\s*[\[(]\s*D\d+(?:\s*[,;]\s*D\d+)*\s*[\])]").expect("Regex")
    });
    let mut refs = refs;
    for m in CITE.find_iter(&answer) {
        refs.extend(m.as_str().split(|c: char| !c.is_alphanumeric()).filter(|t| t.starts_with('D')).map(String::from));
    }
    let clean = CITE.replace_all(&answer, "").into_owned();
    let mut used = HashSet::new();
    let sources = refs
        .iter()
        .filter_map(|r| r.trim().trim_start_matches(['D', 'd']).parse::<usize>().ok())
        .filter(|n| *n >= 1 && *n <= found.len() && used.insert(*n))
        .map(|n| Source {
            id: found[n - 1].id.clone(),
            name: found[n - 1].name.clone(),
            created_at: found[n - 1].created_at,
            tags: doc_tags[n - 1].clone(),
        })
        .collect();
    Ok(Answer {
        answer: if clean.trim().is_empty() { "Dazu habe ich in deinen Dokumenten nichts gefunden.".into() } else { clean.trim().to_string() },
        sources,
        queries: qs,
        model,
        considered: found.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems() {
        assert_eq!(stem("Rechnungen"), "rechnung");
        assert_eq!(stem("Lohnabrechnungen"), "lohnabrechnung");
        assert_eq!(stem("Versicherungen"), "versicherung");
        assert_eq!(stem("Policen"), "polic");
        assert_eq!(stem("Bank"), "bank");
    }

    #[test]
    fn citations_are_removed() {
        let re = regex::Regex::new(r"\s*[\[(]\s*D\d+(?:\s*[,;]\s*D\d+)*\s*[\])]").unwrap();
        assert_eq!(re.replace_all("Nr. 4711 (D1): Bank [D2, D3].", ""), "Nr. 4711: Bank.");
    }

    #[test]
    fn snippets_around_matches() {
        let text = format!("{}Versicherungsnummer: 756.1234.5678.97{}", "x".repeat(5000), "y".repeat(5000));
        let s = snippets(&text, &["versicherungsnummer".into()], 800);
        assert!(s.contains("756.1234.5678.97"));
        assert!(s.chars().count() <= 800);
        assert_eq!(snippets("kurz", &["x".into()], 100), "kurz");
        assert!(snippets(&"a".repeat(3000), &["zzz".into()], 100).starts_with("aaa"));
    }
}

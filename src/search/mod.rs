//! Suche wie bei Papra: freie Wörter plus Filter.
//!
//! | Eingabe | Bedeutung |
//! |---------|-----------|
//! | `rechnung zürich` | beide Wörter (Anfang genügt: `rech` findet „Rechnung“), Umlaute egal |
//! | `"mobile abo"` | genau diese Wortfolge |
//! | `name:swisscom` / `content:iban` | nur im Namen bzw. im Text |
//! | `tag:rechnung`, `tag:"Mein Tag"`, `-tag:privat` | mit bzw. ohne Tag |
//! | `has:tags`, `-has:tags` | mit irgendeinem bzw. ohne Tag |
//! | `type:pdf` / `image` / `text` | Dateiart |
//! | `created:2026`, `created:>2026-01-01`, `created:<=2026-06` | Upload-Datum |
//! | `status:failed` (`pending`, `done`, `skipped`) | Stand der Texterkennung |
//!
//! Alle Werte gehen über `bind` in die Abfrage; der FTS5-Ausdruck wird aus maskierten
//! Begriffen zusammengesetzt, damit Eingaben keine FTS-Syntax einschleusen können.

use crate::db::models::{Document, OcrStatus};
use crate::error::AppResult;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use time::{Date, Month, PrimitiveDateTime, Time};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Query {
    /// Freie Begriffe (Wörter oder "Wortfolgen")
    pub terms: Vec<String>,
    pub name_terms: Vec<String>,
    pub content_terms: Vec<String>,
    pub tags: Vec<String>,
    pub not_tags: Vec<String>,
    pub has_tags: Option<bool>,
    pub kind: Option<Kind>,
    /// Unix-Sekunden, einschliesslich bzw. ausschliesslich
    pub created_from: Option<i64>,
    pub created_until: Option<i64>,
    pub status: Option<OcrStatus>,
    /// Ordner: Dokumente mit mindestens einem dieser Tags (IDs). Nicht Teil der Suchsyntax;
    /// `Some(leer)` heisst: der Ordner hat keine Tags, also keine Dokumente.
    pub any_tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Pdf,
    Image,
    Text,
}

impl Query {
    pub fn is_empty(&self) -> bool {
        *self == Query::default()
    }
}

/// Zerlegt die Eingabe in Teile; Anführungszeichen halten Leerzeichen zusammen.
fn tokens(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in input.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                cur.push(c);
            }
            c if c.is_whitespace() && !quoted => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn unquote(s: &str) -> String {
    s.trim_matches('"').trim().to_string()
}

/// `2026`, `2026-06`, `2026-06-15` → (Beginn, Ende) in Unix-Sekunden (UTC).
fn date_range(s: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = s.split('-').collect();
    let year: i32 = parts.first()?.parse().ok()?;
    let start_end = |y: i32, m: u8, d: u8| -> Option<i64> {
        let date = Date::from_calendar_date(y, Month::try_from(m).ok()?, d).ok()?;
        Some(
            PrimitiveDateTime::new(date, Time::MIDNIGHT)
                .assume_utc()
                .unix_timestamp(),
        )
    };
    match parts.len() {
        1 => Some((start_end(year, 1, 1)?, start_end(year + 1, 1, 1)?)),
        2 => {
            let m: u8 = parts[1].parse().ok()?;
            let (ny, nm) = if m == 12 {
                (year + 1, 1)
            } else {
                (year, m + 1)
            };
            Some((start_end(year, m, 1)?, start_end(ny, nm, 1)?))
        }
        3 => {
            let start = start_end(year, parts[1].parse().ok()?, parts[2].parse().ok()?)?;
            Some((start, start + 86_400))
        }
        _ => None,
    }
}

pub fn parse(input: &str) -> Query {
    let mut q = Query::default();
    for token in tokens(input) {
        let (negated, token) = match token.strip_prefix('-') {
            Some(rest) if rest.contains(':') => (true, rest.to_string()),
            _ => (false, token),
        };
        let Some((key, value)) = token.split_once(':').filter(|(k, _)| !k.starts_with('"')) else {
            let term = unquote(&token);
            if !term.is_empty() {
                q.terms.push(if token.starts_with('"') {
                    format!("\"{term}\"")
                } else {
                    term
                });
            }
            continue;
        };
        let value = unquote(value);
        if value.is_empty() {
            continue;
        }
        match (key.to_lowercase().as_str(), negated) {
            ("tag", false) => q.tags.push(value),
            ("tag", true) => q.not_tags.push(value),
            ("name", _) => q.name_terms.push(value),
            ("content" | "text" | "inhalt", _) => q.content_terms.push(value),
            ("has", neg) if value.eq_ignore_ascii_case("tags") => q.has_tags = Some(!neg),
            ("type" | "typ", _) => {
                q.kind = match value.to_lowercase().as_str() {
                    "pdf" => Some(Kind::Pdf),
                    "image" | "bild" => Some(Kind::Image),
                    "text" | "txt" => Some(Kind::Text),
                    _ => None,
                }
            }
            ("status", _) => q.status = value.to_lowercase().parse().ok(),
            ("created" | "erstellt", _) => {
                let (op, date) = ["<=", ">=", "<", ">", "="]
                    .iter()
                    .find_map(|op| value.strip_prefix(op).map(|d| (*op, d)))
                    .unwrap_or(("=", value.as_str()));
                if let Some((start, end)) = date_range(date) {
                    match op {
                        ">" => q.created_from = Some(end),
                        ">=" => q.created_from = Some(start),
                        "<" => q.created_until = Some(start),
                        "<=" => q.created_until = Some(end),
                        _ => {
                            q.created_from = Some(start);
                            q.created_until = Some(end);
                        }
                    }
                }
            }
            // Unbekannter Filter: als normales Wort suchen
            _ => q.terms.push(unquote(&token)),
        }
    }
    q
}

/// Ein Begriff als FTS5-Ausdruck: immer in Anführungszeichen (keine FTS-Operatoren aus
/// der Eingabe), einzelne Wörter mit Präfix-Suche.
fn fts_term(term: &str) -> Option<String> {
    let phrase = term.starts_with('"') && term.ends_with('"') && term.len() > 1;
    let inner: String = term
        .trim_matches('"')
        .chars()
        .filter(|c| *c != '"')
        .collect();
    let inner = inner.trim();
    if inner.is_empty() || !inner.chars().any(char::is_alphanumeric) {
        return None;
    }
    Some(if phrase || inner.contains(char::is_whitespace) {
        format!("\"{inner}\"")
    } else {
        format!("\"{inner}\"*")
    })
}

/// FTS5-Ausdruck für die Text-Teile der Suche (oder `None`, wenn es keine gibt).
pub fn fts_expression(q: &Query) -> Option<String> {
    let mut parts: Vec<String> = q.terms.iter().filter_map(|t| fts_term(t)).collect();
    parts.extend(
        q.name_terms
            .iter()
            .filter_map(|t| fts_term(t))
            .map(|t| format!("name : {t}")),
    );
    parts.extend(
        q.content_terms
            .iter()
            .filter_map(|t| fts_term(t))
            .map(|t| format!("content : {t}")),
    );
    (!parts.is_empty()).then(|| parts.join(" AND "))
}

const COLUMNS: &str = "d.num, d.id, d.organization_id, d.name, d.original_name, d.mime_type, d.size, d.sha256, d.document_date, '' AS content, d.page_count, d.ocr_status, d.ocr_error, d.source, d.created_by, d.created_at, d.updated_at, d.deleted_at, d.deleted_by";

fn push_filters(b: &mut QueryBuilder<Sqlite>, org: &str, q: &Query) {
    b.push(" WHERE d.organization_id = ")
        .push_bind(org.to_string())
        .push(" AND d.deleted_at IS NULL");
    for tag in &q.tags {
        b.push(" AND EXISTS (SELECT 1 FROM document_tags dt JOIN tags t ON t.id = dt.tag_id WHERE dt.document_id = d.id AND t.name = ")
            .push_bind(tag.clone())
            .push(")");
    }
    if let Some(ids) = &q.any_tag_ids {
        if ids.is_empty() {
            b.push(" AND 0");
        } else {
            b.push(" AND EXISTS (SELECT 1 FROM document_tags dt WHERE dt.document_id = d.id AND dt.tag_id IN (");
            let mut sep = b.separated(", ");
            for id in ids {
                sep.push_bind(id.clone());
            }
            b.push("))");
        }
    }
    for tag in &q.not_tags {
        b.push(" AND NOT EXISTS (SELECT 1 FROM document_tags dt JOIN tags t ON t.id = dt.tag_id WHERE dt.document_id = d.id AND t.name = ")
            .push_bind(tag.clone())
            .push(")");
    }
    match q.has_tags {
        Some(true) => {
            b.push(" AND EXISTS (SELECT 1 FROM document_tags dt WHERE dt.document_id = d.id)");
        }
        Some(false) => {
            b.push(" AND NOT EXISTS (SELECT 1 FROM document_tags dt WHERE dt.document_id = d.id)");
        }
        None => {}
    }
    match q.kind {
        Some(Kind::Pdf) => {
            b.push(" AND d.mime_type = 'application/pdf'");
        }
        Some(Kind::Image) => {
            b.push(" AND d.mime_type LIKE 'image/%'");
        }
        Some(Kind::Text) => {
            b.push(" AND d.mime_type = 'text/plain'");
        }
        None => {}
    }
    if let Some(from) = q.created_from {
        b.push(" AND d.created_at >= ").push_bind(from);
    }
    if let Some(until) = q.created_until {
        b.push(" AND d.created_at < ").push_bind(until);
    }
    if let Some(status) = q.status {
        b.push(" AND d.ocr_status = ").push_bind(status);
    }
}

/// Eine Seite Treffer (bei Text-Suche nach Relevanz, sonst neueste zuerst) und die Gesamtzahl.
pub async fn search(
    pool: &SqlitePool,
    org: &str,
    q: &Query,
    page: u32,
    page_size: u32,
) -> AppResult<(Vec<Document>, i64)> {
    let page_size = page_size.clamp(1, 200);
    let fts = fts_expression(q);

    let mut b: QueryBuilder<Sqlite> =
        QueryBuilder::new(format!("SELECT {COLUMNS} FROM documents d"));
    if let Some(expr) = &fts {
        b.push(" JOIN (SELECT rowid, bm25(documents_fts, 10.0, 1.0) AS rank FROM documents_fts WHERE documents_fts MATCH ")
            .push_bind(expr.clone())
            .push(") f ON f.rowid = d.num");
    }
    push_filters(&mut b, org, q);
    b.push(if fts.is_some() {
        " ORDER BY f.rank, d.created_at DESC"
    } else {
        " ORDER BY d.created_at DESC, d.num DESC"
    });
    b.push(" LIMIT ")
        .push_bind(page_size as i64)
        .push(" OFFSET ")
        .push_bind(page as i64 * page_size as i64);
    let docs = b.build_query_as::<Document>().fetch_all(pool).await?;

    let mut c: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT count(*) FROM documents d");
    if let Some(expr) = &fts {
        c.push(" JOIN (SELECT rowid FROM documents_fts WHERE documents_fts MATCH ")
            .push_bind(expr.clone())
            .push(") f ON f.rowid = d.num");
    }
    push_filters(&mut c, org, q);
    let total: i64 = c.build_query_scalar().fetch_one(pool).await?;
    Ok((docs, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_filters_and_terms() {
        let q = parse(
            r#"rechnung "mobile abo" tag:Steuern tag:"Mein Tag" -tag:privat name:swisscom type:pdf has:tags status:failed"#,
        );
        assert_eq!(q.terms, vec!["rechnung", "\"mobile abo\""]);
        assert_eq!(q.tags, vec!["Steuern", "Mein Tag"]);
        assert_eq!(q.not_tags, vec!["privat"]);
        assert_eq!(q.name_terms, vec!["swisscom"]);
        assert_eq!(q.kind, Some(Kind::Pdf));
        assert_eq!(q.has_tags, Some(true));
        assert_eq!(q.status, Some(OcrStatus::Failed));
        assert_eq!(parse("-has:tags").has_tags, Some(false));
        assert!(parse("   ").is_empty());
    }

    #[test]
    fn parses_dates() {
        let y2026 = 1_767_225_600; // 2026-01-01T00:00:00Z
        assert_eq!(parse("created:2026").created_from, Some(y2026));
        assert_eq!(parse("created:>=2026-01-01").created_from, Some(y2026));
        assert_eq!(parse("created:>2025-12-31").created_from, Some(y2026));
        assert_eq!(parse("created:<2026").created_until, Some(y2026));
        let june = parse("created:2026-06");
        assert_eq!(
            june.created_until.unwrap() - june.created_from.unwrap(),
            30 * 86_400
        );
        assert_eq!(parse("created:kaputt").created_from, None);
    }

    #[test]
    fn fts_expression_is_escaped() {
        let q = parse(r#"zürich "mobile abo" name:swiss content:iban"#);
        assert_eq!(
            fts_expression(&q).unwrap(),
            r#""zürich"* AND "mobile abo" AND name : "swiss"* AND content : "iban"*"#
        );
        // FTS-Operatoren und Anführungszeichen aus der Eingabe werden zu harmlosem Text:
        // jeder Teil steht vollständig in Anführungszeichen, ohne innere Anführungszeichen.
        for input in [
            r#"a*" OR content:x NEAR( ^"#,
            "foo OR bar",
            "x AND NOT y",
            "{col}: z",
            r#"""""#,
        ] {
            if let Some(expr) = fts_expression(&parse(input)) {
                for part in expr.split(" AND ") {
                    let part = part
                        .trim_start_matches("name : ")
                        .trim_start_matches("content : ")
                        .trim_end_matches('*');
                    assert!(
                        part.starts_with('"')
                            && part.ends_with('"')
                            && !part[1..part.len() - 1].contains('"'),
                        "{input} → {expr}"
                    );
                }
            }
        }
        assert_eq!(fts_expression(&parse("tag:x")), None);
        assert_eq!(fts_term("***"), None);
    }
}

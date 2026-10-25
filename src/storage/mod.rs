//! Dokument-Inhalte als Blöcke (1 MiB) in der verschlüsselten Datenbank. Beim Ausliefern
//! werden nur die Blöcke gelesen, die eine Range-Anfrage braucht – so lädt die PDF-Vorschau
//! grosse Dateien stückweise, ohne sie ganz in den Speicher zu holen.

use crate::db::models::Document;
use crate::error::AppResult;
use rocket::http::{ContentType, Header, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder, Response};
use sqlx::{SqliteConnection, SqlitePool};
use std::io::Cursor;

pub const CHUNK_SIZE: usize = 1024 * 1024;

/// Datei in Blöcken schreiben (innerhalb der Transaktion, die auch das Dokument anlegt).
pub async fn write(conn: &mut SqliteConnection, document_id: &str, data: &[u8]) -> AppResult<()> {
    for (seq, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
        sqlx::query("INSERT INTO document_chunks (document_id, seq, data) VALUES (?, ?, ?)")
            .bind(document_id)
            .bind(seq as i64)
            .bind(chunk)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

pub async fn read_all(pool: &SqlitePool, document_id: &str) -> AppResult<Vec<u8>> {
    let parts: Vec<Vec<u8>> =
        sqlx::query_scalar("SELECT data FROM document_chunks WHERE document_id = ? ORDER BY seq")
            .bind(document_id)
            .fetch_all(pool)
            .await?;
    Ok(parts.concat())
}

/// Bytes `start..=end` lesen – nur die betroffenen Blöcke.
pub async fn read_range(
    pool: &SqlitePool,
    document_id: &str,
    start: u64,
    end: u64,
) -> AppResult<Vec<u8>> {
    let size = CHUNK_SIZE as u64;
    let (first, last) = (start / size, end / size);
    let parts: Vec<Vec<u8>> = sqlx::query_scalar("SELECT data FROM document_chunks WHERE document_id = ? AND seq BETWEEN ? AND ? ORDER BY seq")
        .bind(document_id)
        .bind(first as i64)
        .bind(last as i64)
        .fetch_all(pool)
        .await?;
    let joined = parts.concat();
    let offset = (start - first * size) as usize;
    let len = (end - start + 1) as usize;
    Ok(joined
        .get(offset..offset + len)
        .map(<[u8]>::to_vec)
        .unwrap_or_default())
}

/// Einzelner Bereich aus `Range: bytes=…`. `None` = ganze Datei, `Err` = nicht erfüllbar (416).
pub fn parse_range(header: Option<&str>, size: u64) -> Result<Option<(u64, u64)>, ()> {
    let Some(spec) = header.and_then(|h| h.trim().strip_prefix("bytes=")) else {
        return Ok(None);
    };
    if spec.contains(',') || size == 0 {
        return Ok(None); // Mehrfach-Bereiche: einfach die ganze Datei schicken
    }
    let (a, b) = spec.split_once('-').ok_or(())?;
    let (start, end) = match (a.trim(), b.trim()) {
        ("", n) => {
            let n: u64 = n.parse().map_err(|_| ())?;
            if n == 0 {
                return Err(());
            }
            (size.saturating_sub(n), size - 1)
        }
        (s, "") => (s.parse().map_err(|_| ())?, size - 1),
        (s, e) => (
            s.parse().map_err(|_| ())?,
            e.parse::<u64>().map_err(|_| ())?.min(size - 1),
        ),
    };
    if start > end || start >= size {
        return Err(());
    }
    Ok(Some((start, end)))
}

/// Der `Range`-Header einer Anfrage.
pub struct RangeHeader(pub Option<String>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RangeHeader {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(RangeHeader(
            req.headers().get_one("Range").map(String::from),
        ))
    }
}

/// Datei-Antwort mit Range-Unterstützung (206), inline oder als Download.
pub struct FileResponse {
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    pub range: Option<(u64, u64)>,
    pub body: Vec<u8>,
    pub attachment: bool,
}

impl FileResponse {
    /// Liest genau das, was die Anfrage braucht. Unerfüllbarer Bereich → `Err(Dateigrösse)` (für 416).
    pub async fn load(
        pool: &SqlitePool,
        doc: &Document,
        range_header: Option<&str>,
        attachment: bool,
    ) -> AppResult<Result<Self, u64>> {
        let size = doc.size as u64;
        let Ok(range) = parse_range(range_header, size) else {
            return Ok(Err(size));
        };
        let body = match range {
            Some((start, end)) => read_range(pool, &doc.id, start, end).await?,
            None => read_all(pool, &doc.id).await?,
        };
        Ok(Ok(FileResponse {
            name: doc.name.clone(),
            mime_type: doc.mime_type.clone(),
            size,
            range,
            body,
            attachment,
        }))
    }
}

/// `Content-Disposition` mit ASCII-Ersatz und UTF-8-Namen (RFC 6266/5987).
fn disposition(name: &str, attachment: bool) -> String {
    let ascii: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_graphic() && c != '"' && c != '\\' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let encoded: String = name
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    format!(
        "{}; filename=\"{ascii}\"; filename*=UTF-8''{encoded}",
        if attachment { "attachment" } else { "inline" }
    )
}

impl<'r> Responder<'r, 'static> for FileResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let content_type =
            ContentType::parse_flexible(&self.mime_type).unwrap_or(ContentType::Binary);
        let mut res = Response::build();
        res.header(content_type)
            .header(Header::new("Accept-Ranges", "bytes"))
            .header(Header::new(
                "Content-Disposition",
                disposition(&self.name, self.attachment),
            ))
            // Vertrauliche Inhalte: nicht in geteilten Caches ablegen.
            .header(Header::new("Cache-Control", "private, no-store"));
        if let Some((start, end)) = self.range {
            res.status(Status::PartialContent).header(Header::new(
                "Content-Range",
                format!("bytes {start}-{end}/{}", self.size),
            ));
        }
        res.sized_body(self.body.len(), Cursor::new(self.body)).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges() {
        assert_eq!(parse_range(None, 100), Ok(None));
        assert_eq!(parse_range(Some("bytes=0-9"), 100), Ok(Some((0, 9))));
        assert_eq!(parse_range(Some("bytes=90-"), 100), Ok(Some((90, 99))));
        assert_eq!(parse_range(Some("bytes=-10"), 100), Ok(Some((90, 99))));
        assert_eq!(parse_range(Some("bytes=50-500"), 100), Ok(Some((50, 99))));
        assert_eq!(parse_range(Some("bytes=0-1,5-6"), 100), Ok(None));
        assert_eq!(parse_range(Some("bytes=100-"), 100), Err(()));
        assert_eq!(parse_range(Some("bytes=9-3"), 100), Err(()));
        assert_eq!(parse_range(Some("bytes=-0"), 100), Err(()));
        assert_eq!(parse_range(Some("bytes=x-y"), 100), Err(()));
    }

    #[test]
    fn disposition_keeps_umlauts_safe() {
        let d = disposition("Rechnung Zürich \"neu\".pdf", false);
        assert!(d.starts_with("inline; filename=\"Rechnung Z_rich _neu_.pdf\""));
        assert!(d.contains("filename*=UTF-8''Rechnung%20Z%C3%BCrich%20%22neu%22.pdf"));
    }
}

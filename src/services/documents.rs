//! Dokumente: aufnehmen (Web, API, Eingangsordner), lesen, ändern, Papierkorb.

use crate::db::models::{new_id, now, Document, DocumentSource, OcrStatus, WebhookEvent};
use crate::error::{AppError, AppResult};
use crate::ocr::{engine::is_ocr_image, OcrQueue};
use crate::services::webhooks;
use crate::storage;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

/// Spalten für Listen – ohne den (evtl. langen) Inhalt.
const LIST_COLUMNS: &str = "num, id, organization_id, name, original_name, mime_type, size, sha256, document_date, '' AS content, page_count, ocr_status, ocr_error, source, created_by, created_at, updated_at, deleted_at, deleted_by";

pub struct NewDocument<'a> {
    pub organization_id: &'a str,
    pub user_id: Option<&'a str>,
    pub source: DocumentSource,
    pub file_name: &'a str,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum Ingested {
    Created(Document),
    /// Dieselbe Datei lag im Papierkorb und ist wieder da.
    Restored(Document),
}

impl Ingested {
    pub fn document(&self) -> &Document {
        match self {
            Ingested::Created(d) | Ingested::Restored(d) => d,
        }
    }
}

/// Nutzlast für Webhooks.
pub fn payload(doc: &Document) -> serde_json::Value {
    serde_json::json!({
        "documentId": doc.id,
        "organizationId": doc.organization_id,
        "name": doc.name,
        "mimeType": doc.mime_type,
        "size": doc.size,
        "ocrStatus": doc.ocr_status,
        "createdAt": doc.created_at,
    })
}

async fn emit(pool: &SqlitePool, doc: &Document, event: WebhookEvent) {
    webhooks::emit(pool, &doc.organization_id, event, payload(doc)).await;
}

/// Dateiname ohne Pfad und Steuerzeichen, höchstens 255 Zeichen.
pub fn clean_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let cleaned: String = base
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .chars()
        .take(255)
        .collect();
    if cleaned.is_empty() {
        "Dokument".into()
    } else {
        cleaned
    }
}

/// Dateityp aus dem Inhalt (nicht aus dem Namen); Text erkennt man an gültigem UTF-8.
pub fn detect_mime(data: &[u8]) -> String {
    if let Some(kind) = infer::get(data) {
        return kind.mime_type().to_string();
    }
    if !data.is_empty() && std::str::from_utf8(data).is_ok() {
        return "text/plain".into();
    }
    "application/octet-stream".into()
}

/// Neue Datei aufnehmen: Typ, Prüfsumme, Duplikat-Prüfung, Blöcke schreiben, OCR anstossen.
pub async fn ingest(
    pool: &SqlitePool,
    queue: &OcrQueue,
    new: NewDocument<'_>,
) -> AppResult<Ingested> {
    if new.data.is_empty() {
        return Err(AppError::bad("Die Datei ist leer."));
    }
    let sha256 = hex::encode(Sha256::digest(&new.data));
    let existing: Option<Document> =
        sqlx::query_as("SELECT * FROM documents WHERE organization_id = ? AND sha256 = ?")
            .bind(new.organization_id)
            .bind(&sha256)
            .fetch_optional(pool)
            .await?;
    if let Some(doc) = existing {
        if !doc.in_trash() {
            return Err(AppError::Conflict(format!(
                "Dieses Dokument gibt es schon: „{}“.",
                doc.name
            )));
        }
        sqlx::query("UPDATE documents SET deleted_at = NULL, deleted_by = NULL, updated_at = ? WHERE id = ?").bind(now()).bind(&doc.id).execute(pool).await?;
        log(
            pool,
            &doc.id,
            "restored",
            new.user_id,
            serde_json::json!({ "reason": "erneut hochgeladen" }),
        )
        .await?;
        let doc = get(pool, new.organization_id, &doc.id).await?;
        emit(pool, &doc, WebhookEvent::DocumentRestored).await;
        return Ok(Ingested::Restored(doc));
    }

    let mime = detect_mime(&new.data);
    let name = clean_name(new.file_name);
    // Text-Dateien brauchen keine Erkennung, andere Typen können wir nicht lesen.
    let (status, content) = if mime == "application/pdf" || is_ocr_image(&mime) {
        (OcrStatus::Pending, String::new())
    } else if mime == "text/plain" {
        (
            OcrStatus::Done,
            String::from_utf8_lossy(&new.data)
                .chars()
                .take(2_000_000)
                .collect(),
        )
    } else {
        (OcrStatus::Skipped, String::new())
    };

    let id = new_id();
    let ts = now();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO documents (id, organization_id, name, original_name, mime_type, size, sha256, content, ocr_status, source, created_by, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(new.organization_id)
    .bind(&name)
    .bind(&name)
    .bind(&mime)
    .bind(new.data.len() as i64)
    .bind(&sha256)
    .bind(&content)
    .bind(status)
    .bind(new.source)
    .bind(new.user_id)
    .bind(ts)
    .bind(ts)
    .execute(&mut *tx)
    .await?;
    storage::write(&mut tx, &id, &new.data).await?;
    tx.commit().await?;
    log(
        pool,
        &id,
        "created",
        new.user_id,
        serde_json::json!({ "source": new.source, "size": new.data.len() }),
    )
    .await?;
    if status == OcrStatus::Pending {
        queue.wake();
    } else if let Err(e) =
        crate::services::tagging_rules::apply_to_document(pool, new.organization_id, &id).await
    {
        // Text-Dateien und unlesbare Typen: der Text steht sofort fest.
        tracing::warn!(document = %id, "Tagging-Regeln: {e}");
    }
    tracing::info!(document = %id, organization = new.organization_id, mime = %mime, size = new.data.len(), source = %new.source, "Dokument aufgenommen");
    let doc = get(pool, new.organization_id, &id).await?;
    emit(pool, &doc, WebhookEvent::DocumentCreated).await;
    Ok(Ingested::Created(doc))
}

/// Dokument einer Organisation (auch im Papierkorb) – sonst 404.
pub async fn get(pool: &SqlitePool, organization_id: &str, id: &str) -> AppResult<Document> {
    sqlx::query_as("SELECT * FROM documents WHERE id = ? AND organization_id = ?")
        .bind(id)
        .bind(organization_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

/// Seite einer Liste (neueste zuerst), ohne Inhalt; dazu die Gesamtzahl.
pub async fn list(
    pool: &SqlitePool,
    organization_id: &str,
    trash: bool,
    page: u32,
    page_size: u32,
) -> AppResult<(Vec<Document>, i64)> {
    let page_size = page_size.clamp(1, 200);
    let filter = if trash {
        "deleted_at IS NOT NULL"
    } else {
        "deleted_at IS NULL"
    };
    let order = if trash {
        "deleted_at DESC"
    } else {
        "created_at DESC, num DESC"
    };
    // Spalten, Filter und Sortierung sind feste Bausteine; Werte gehen über bind.
    let sql = format!("SELECT {LIST_COLUMNS} FROM documents WHERE organization_id = ? AND {filter} ORDER BY {order} LIMIT ? OFFSET ?");
    let docs = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(organization_id)
        .bind(page_size as i64)
        .bind((page as i64) * page_size as i64)
        .fetch_all(pool)
        .await?;
    let total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT count(*) FROM documents WHERE organization_id = ? AND {filter}"
    )))
    .bind(organization_id)
    .fetch_one(pool)
    .await?;
    Ok((docs, total))
}

pub async fn rename(
    pool: &SqlitePool,
    organization_id: &str,
    id: &str,
    name: &str,
    user_id: Option<&str>,
) -> AppResult<Document> {
    let doc = get(pool, organization_id, id).await?;
    let name = clean_name(name);
    sqlx::query("UPDATE documents SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now())
        .bind(id)
        .execute(pool)
        .await?;
    log(
        pool,
        id,
        "renamed",
        user_id,
        serde_json::json!({ "from": doc.name, "to": name }),
    )
    .await?;
    let doc = get(pool, organization_id, id).await?;
    emit(pool, &doc, WebhookEvent::DocumentUpdated).await;
    Ok(doc)
}

/// Erkannten Text von Hand ersetzen (fliesst in die Suche).
pub async fn set_content(
    pool: &SqlitePool,
    organization_id: &str,
    id: &str,
    content: &str,
    user_id: Option<&str>,
) -> AppResult<Document> {
    get(pool, organization_id, id).await?;
    sqlx::query("UPDATE documents SET content = ?, updated_at = ? WHERE id = ?")
        .bind(content)
        .bind(now())
        .bind(id)
        .execute(pool)
        .await?;
    log(pool, id, "content_updated", user_id, serde_json::json!({})).await?;
    let doc = get(pool, organization_id, id).await?;
    emit(pool, &doc, WebhookEvent::DocumentUpdated).await;
    Ok(doc)
}

/// Texterkennung erneut ausführen.
pub async fn reprocess(
    pool: &SqlitePool,
    queue: &OcrQueue,
    organization_id: &str,
    id: &str,
) -> AppResult<()> {
    get(pool, organization_id, id).await?;
    sqlx::query("UPDATE documents SET ocr_status = 'pending', ocr_error = NULL WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    queue.wake();
    Ok(())
}

/// In den Papierkorb.
pub async fn trash(
    pool: &SqlitePool,
    organization_id: &str,
    id: &str,
    user_id: Option<&str>,
) -> AppResult<()> {
    let doc = get(pool, organization_id, id).await?;
    if doc.in_trash() {
        return Ok(());
    }
    sqlx::query("UPDATE documents SET deleted_at = ?, deleted_by = ?, updated_at = ? WHERE id = ?")
        .bind(now())
        .bind(user_id)
        .bind(now())
        .bind(id)
        .execute(pool)
        .await?;
    log(pool, id, "deleted", user_id, serde_json::json!({})).await?;
    emit(pool, &doc, WebhookEvent::DocumentDeleted).await;
    Ok(())
}

pub async fn restore(
    pool: &SqlitePool,
    organization_id: &str,
    id: &str,
    user_id: Option<&str>,
) -> AppResult<()> {
    let doc = get(pool, organization_id, id).await?;
    if !doc.in_trash() {
        return Ok(());
    }
    sqlx::query(
        "UPDATE documents SET deleted_at = NULL, deleted_by = NULL, updated_at = ? WHERE id = ?",
    )
    .bind(now())
    .bind(id)
    .execute(pool)
    .await?;
    log(pool, id, "restored", user_id, serde_json::json!({})).await?;
    emit(pool, &doc, WebhookEvent::DocumentRestored).await;
    Ok(())
}

/// Endgültig löschen – nur aus dem Papierkorb. Blöcke und Suchindex verschwinden mit (Löschkette, Trigger).
pub async fn delete_permanently(
    pool: &SqlitePool,
    organization_id: &str,
    id: &str,
) -> AppResult<()> {
    let doc = get(pool, organization_id, id).await?;
    if !doc.in_trash() {
        return Err(AppError::bad(
            "Nur Dokumente im Papierkorb können endgültig gelöscht werden.",
        ));
    }
    sqlx::query("DELETE FROM documents WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Papierkorb leeren: alles, was länger als `retention_days` darin liegt.
pub async fn purge_trash(pool: &SqlitePool, retention_days: u32) -> AppResult<u64> {
    let limit = now() - i64::from(retention_days) * 86_400;
    Ok(
        sqlx::query("DELETE FROM documents WHERE deleted_at IS NOT NULL AND deleted_at < ?")
            .bind(limit)
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

pub async fn log(
    pool: &SqlitePool,
    document_id: &str,
    event: &str,
    user_id: Option<&str>,
    data: serde_json::Value,
) -> AppResult<()> {
    sqlx::query("INSERT INTO document_activity (id, document_id, event, user_id, data, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(new_id())
        .bind(document_id)
        .bind(event)
        .bind(user_id)
        .bind(data.to_string())
        .bind(now())
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_types() {
        assert_eq!(clean_name("C:\\Users\\x\\Rechnung.pdf"), "Rechnung.pdf");
        assert_eq!(clean_name("../../etc/passwd"), "passwd");
        assert_eq!(clean_name("  \u{7}  "), "Dokument");
        assert_eq!(detect_mime(b"%PDF-1.4\n..."), "application/pdf");
        assert_eq!(
            detect_mime(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0]),
            "image/png"
        );
        assert_eq!(detect_mime("Grüsse".as_bytes()), "text/plain");
        assert_eq!(
            detect_mime(&[0xff, 0xfe, 0x00, 0x81]),
            "application/octet-stream"
        );
    }
}

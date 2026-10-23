//! OCR-Warteschlange. Der Zustand liegt in der Datenbank (`documents.ocr_status`), damit
//! nach einem Neustart nichts verloren geht; ein `Notify` weckt die Worker sofort, wenn
//! neue Dokumente kommen (sonst schauen sie alle 30 Sekunden nach).

use super::engine::Engine;
use crate::db::models::{now, OcrStatus};
use crate::error::AppResult;
use crate::storage;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

#[derive(Clone, Default)]
pub struct OcrQueue {
    notify: Arc<Notify>,
}

impl OcrQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Neue Arbeit: wartende Worker wecken (oder den nächsten, der wartet).
    pub fn wake(&self) {
        self.notify.notify_waiters();
        self.notify.notify_one();
    }

    async fn wait(&self, timeout: Duration) {
        let _ = tokio::time::timeout(timeout, self.notify.notified()).await;
    }
}

/// Nach einem Absturz: halb bearbeitete Dokumente wieder in die Warteschlange.
pub async fn recover(pool: &SqlitePool) -> AppResult<u64> {
    Ok(
        sqlx::query("UPDATE documents SET ocr_status = 'pending' WHERE ocr_status = 'processing'")
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

/// Nächstes Dokument atomar übernehmen – mehrere Worker greifen nie dasselbe.
pub async fn claim_next(pool: &SqlitePool) -> AppResult<Option<String>> {
    Ok(sqlx::query_scalar(
        "UPDATE documents SET ocr_status = 'processing'
         WHERE num = (SELECT num FROM documents WHERE ocr_status = 'pending' AND deleted_at IS NULL ORDER BY num LIMIT 1)
         RETURNING id",
    )
    .fetch_optional(pool)
    .await?)
}

/// Ein Dokument erkennen und Text, Seitenzahl und Status speichern.
pub async fn process(
    pool: &SqlitePool,
    engine: &Engine,
    document_id: &str,
) -> AppResult<OcrStatus> {
    let mime: String = sqlx::query_scalar("SELECT mime_type FROM documents WHERE id = ?")
        .bind(document_id)
        .fetch_one(pool)
        .await?;
    let data = storage::read_all(pool, document_id).await?;
    let started = std::time::Instant::now();
    let (status, text, pages, error) = match engine.extract(&mime, &data).await {
        Ok(Some(e)) => (OcrStatus::Done, Some(e.text), e.pages, None),
        Ok(None) => (OcrStatus::Skipped, None, None, None),
        Err(e) => (OcrStatus::Failed, None, None, Some(e)),
    };
    // Inhalt nur überschreiben, wenn etwas erkannt wurde (manuell bearbeiteter Text bleibt sonst).
    sqlx::query(
        "UPDATE documents SET ocr_status = ?, ocr_error = ?, content = COALESCE(?, content), page_count = COALESCE(?, page_count), updated_at = ?
         WHERE id = ?",
    )
    .bind(status)
    .bind(&error)
    .bind(&text)
    .bind(pages)
    .bind(now())
    .bind(document_id)
    .execute(pool)
    .await?;
    match &error {
        Some(e) => tracing::warn!(document = document_id, "Texterkennung fehlgeschlagen: {e}"),
        None => tracing::info!(
            document = document_id,
            ms = started.elapsed().as_millis() as u64,
            chars = text.as_ref().map(|t| t.chars().count()).unwrap_or(0),
            "Texterkennung {status}"
        ),
    }
    Ok(status)
}

/// Startet `workers` Worker, die laufen, bis der Server stoppt.
pub fn spawn_workers(
    pool: SqlitePool,
    engine: Arc<Engine>,
    queue: OcrQueue,
    workers: usize,
    shutdown: rocket::Shutdown,
) {
    for n in 0..workers.max(1) {
        let (pool, engine, queue, mut shutdown) = (
            pool.clone(),
            engine.clone(),
            queue.clone(),
            shutdown.clone(),
        );
        tokio::spawn(async move {
            loop {
                match claim_next(&pool).await {
                    Ok(Some(id)) => {
                        if let Err(e) = process(&pool, &engine, &id).await {
                            tracing::error!(worker = n, document = %id, "OCR-Worker: {e}");
                        }
                        continue;
                    }
                    Ok(None) => {}
                    Err(e) => tracing::error!(worker = n, "OCR-Warteschlange: {e}"),
                }
                tokio::select! {
                    _ = queue.wait(Duration::from_secs(30)) => {}
                    _ = &mut shutdown => break,
                }
            }
        });
    }
}

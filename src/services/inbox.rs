//! Prüfliste (übernommen aus nrzDocsScanner, Ziel ist jetzt nrzPaper selbst): Scans und
//! importierte PDFs warten hier, bis die KI Name, Tags und Metadaten vorgeschlagen hat und jemand
//! freigibt. Erst die Freigabe legt ein Dokument an – mit Tags (neue werden angelegt) und den
//! Metadaten als Eigenschaften. Danach werden PDF und Seitenbilder aus der Prüfliste gelöscht.

use crate::ai::{self, AiConfig};
use crate::db::models::{new_id, now, DocumentSource, InboxItem, InboxStatus};
use crate::error::{AppError, AppResult};
use crate::ocr::OcrQueue;
use crate::services::documents::{self, NewDocument};
use crate::services::{properties, tags};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::BTreeMap;

/// Name ohne Pfadzeichen und ohne ".pdf"; leer → "Scan <Datum>".
pub fn clean_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if "/\\:*?\"<>|\n\r\t".contains(c) {
                '-'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim();
    let cleaned = cleaned
        .strip_suffix(".pdf")
        .or_else(|| cleaned.strip_suffix(".PDF"))
        .unwrap_or(cleaned)
        .trim();
    let cleaned: String = cleaned.chars().take(200).collect();
    if cleaned.is_empty() {
        let d = time::OffsetDateTime::now_utc();
        format!("Scan {:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
    } else {
        cleaned
    }
}

/// Neuer Eintrag (PDF plus ggf. die Seiten als JPEG); die KI-Analyse startet im Hintergrund.
/// `auto_name`: die KI darf den Namen ersetzen (kein eigener eingegeben).
#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &SqlitePool,
    ai: &AiConfig,
    org: &str,
    user: Option<&str>,
    name: &str,
    auto_name: bool,
    source: &str,
    source_info: &str,
    hint_tags: Vec<String>,
    pdf: Vec<u8>,
    pages: Vec<Vec<u8>>,
) -> AppResult<InboxItem> {
    if !pdf.starts_with(b"%PDF") {
        return Err(AppError::bad("Das ist keine PDF-Datei."));
    }
    let id = new_id();
    let ts = now();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO inbox_items (id, organization_id, user_id, name, auto_name, status, step, tags, page_count, source, source_info, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'analyse', 'KI liest das Dokument …', ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(org)
    .bind(user)
    .bind(clean_name(name))
    .bind(auto_name)
    .bind(sqlx::types::Json(&hint_tags))
    .bind(pages.len() as i64)
    .bind(source)
    .bind(source_info.chars().take(300).collect::<String>())
    .bind(ts)
    .bind(ts)
    .execute(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO inbox_files (inbox_id, kind, position, data) VALUES (?, 'pdf', 0, ?)")
        .bind(&id)
        .bind(pdf)
        .execute(&mut *tx)
        .await?;
    for (i, page) in pages.into_iter().enumerate() {
        sqlx::query(
            "INSERT INTO inbox_files (inbox_id, kind, position, data) VALUES (?, 'page', ?, ?)",
        )
        .bind(&id)
        .bind(i as i64 + 1)
        .bind(page)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    tokio::spawn(analyze(pool.clone(), ai.clone(), id.clone()));
    get(pool, org, &id).await
}

pub async fn get(pool: &SqlitePool, org: &str, id: &str) -> AppResult<InboxItem> {
    sqlx::query_as("SELECT * FROM inbox_items WHERE id = ? AND organization_id = ?")
        .bind(id)
        .bind(org)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

/// Offene Einträge (neueste zuerst) und die zuletzt abgelegten.
pub async fn list(pool: &SqlitePool, org: &str) -> AppResult<Vec<InboxItem>> {
    Ok(sqlx::query_as(
        "SELECT * FROM (
           SELECT * FROM inbox_items WHERE organization_id = ? AND status != 'fertig'
           UNION ALL
           SELECT * FROM (SELECT * FROM inbox_items WHERE organization_id = ? AND status = 'fertig' ORDER BY updated_at DESC LIMIT 15)
         ) ORDER BY status = 'fertig', created_at DESC",
    )
    .bind(org)
    .bind(org)
    .fetch_all(pool)
    .await?)
}

/// Zähler für die Navigation: was auf Prüfung wartet oder gerade gelesen wird.
pub async fn open_count(pool: &SqlitePool, org: &str) -> AppResult<i64> {
    Ok(sqlx::query_scalar("SELECT count(*) FROM inbox_items WHERE organization_id = ? AND status IN ('analyse', 'pruefen', 'fehler')")
        .bind(org)
        .fetch_one(pool)
        .await?)
}

/// PDF (`page` = 0) oder Seitenbild (1, 2, …).
pub async fn file(pool: &SqlitePool, org: &str, id: &str, page: i64) -> AppResult<Vec<u8>> {
    get(pool, org, id).await?;
    let kind = if page == 0 { "pdf" } else { "page" };
    sqlx::query_scalar(
        "SELECT data FROM inbox_files WHERE inbox_id = ? AND kind = ? AND position = ?",
    )
    .bind(id)
    .bind(kind)
    .bind(page)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Änderungen aus der Prüfung. Fehlt ein Feld, bleibt es, wie es ist.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub name: Option<String>,
    /// In eine andere Organisation verschieben (der Aufrufer prüft die Mitgliedschaft).
    pub organization_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<BTreeMap<String, String>>,
    pub summary: Option<String>,
}

pub async fn edit(pool: &SqlitePool, org: &str, id: &str, e: Edit) -> AppResult<InboxItem> {
    let mut item = get(pool, org, id).await?;
    if !item.editable() {
        return Err(AppError::Conflict(
            "Dieses Dokument wird gerade bearbeitet oder ist schon abgelegt.".into(),
        ));
    }
    if let Some(v) = e.name {
        item.name = clean_name(&v);
        item.auto_name = false;
    }
    if let Some(target) = e.organization_id.filter(|t| t != &item.organization_id) {
        // Andere Organisation, andere Tags.
        item.organization_id = target;
        item.tags.0.clear();
    }
    if let Some(v) = e.tags {
        let mut tags: Vec<String> = Vec::new();
        for t in v
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
        {
            if !tags.iter().any(|x| x.eq_ignore_ascii_case(&t)) {
                tags.push(t);
            }
        }
        item.tags.0 = tags;
    }
    if let Some(v) = e.metadata {
        item.metadata.0 = v
            .into_iter()
            .filter(|(k, v)| {
                !v.trim().is_empty() && ai::META_FIELDS.iter().any(|(key, _)| key == k)
            })
            .map(|(k, v)| (k, v.trim().to_string()))
            .collect();
    }
    if let Some(v) = e.summary {
        item.summary = v.trim().to_string();
    }
    sqlx::query(
        "UPDATE inbox_items SET name = ?, auto_name = ?, organization_id = ?, tags = ?, metadata = ?, summary = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&item.name)
    .bind(item.auto_name)
    .bind(&item.organization_id)
    .bind(&item.tags)
    .bind(&item.metadata)
    .bind(&item.summary)
    .bind(now())
    .bind(id)
    .execute(pool)
    .await?;
    get(pool, &item.organization_id, id).await
}

async fn set_status(
    pool: &SqlitePool,
    id: &str,
    status: InboxStatus,
    step: &str,
    error: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE inbox_items SET status = ?, step = ?, error = ?, updated_at = ? WHERE id = ?",
    )
    .bind(status)
    .bind(step)
    .bind(error)
    .bind(now())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Neu analysieren lassen (nur in der Prüfung oder nach einem Fehler).
pub async fn reanalyze(pool: &SqlitePool, ai: &AiConfig, org: &str, id: &str) -> AppResult<()> {
    let item = get(pool, org, id).await?;
    if !item.editable() {
        return Err(AppError::Conflict(
            "Das Dokument wird gerade bearbeitet.".into(),
        ));
    }
    set_status(
        pool,
        id,
        InboxStatus::Analyse,
        "KI liest das Dokument …",
        "",
    )
    .await?;
    tokio::spawn(analyze(pool.clone(), ai.clone(), id.to_string()));
    Ok(())
}

/// KI liest Name, Tags und Metadaten – das Ergebnis wartet danach auf die Freigabe.
/// Scheitert die KI, bleibt der Eintrag mit Hinweis prüfbar (Name und Tags dann selbst setzen).
pub async fn analyze(pool: SqlitePool, ai: AiConfig, id: String) {
    let result = async {
        let item: InboxItem = sqlx::query_as("SELECT * FROM inbox_items WHERE id = ?").bind(&id).fetch_one(&pool).await?;
        let org_name: String =
            sqlx::query_scalar("SELECT name FROM organizations WHERE id = ?").bind(&item.organization_id).fetch_one(&pool).await?;
        let tag_infos: Vec<ai::TagInfo> = tags::list(&pool, &item.organization_id)
            .await?
            .into_iter()
            .map(|(t, _)| ai::TagInfo { name: t.name, description: t.description })
            .collect();
        let filename = format!("{}.pdf", item.name);
        let pages: Vec<Vec<u8>> =
            sqlx::query_scalar("SELECT data FROM inbox_files WHERE inbox_id = ? AND kind = 'page' ORDER BY position LIMIT 4")
                .bind(&id)
                .fetch_all(&pool)
                .await?;
        let analysis = if pages.is_empty() {
            let pdf: Vec<u8> = sqlx::query_scalar("SELECT data FROM inbox_files WHERE inbox_id = ? AND kind = 'pdf'")
                .bind(&id)
                .fetch_one(&pool)
                .await?;
            ai::analyze(&ai, &org_name, &filename, ai::Input::Pdf(&pdf), &tag_infos).await
        } else {
            ai::analyze(&ai, &org_name, &filename, ai::Input::Pages(&pages), &tag_infos).await
        };
        Ok::<_, AppError>((item, analysis))
    }
    .await;

    let outcome = match result {
        Ok((mut item, Ok(a))) => {
            // Ein selbst eingegebener Name hat Vorrang vor dem KI-Vorschlag.
            if item.auto_name && !a.title.is_empty() {
                item.name = clean_name(&a.title);
            }
            for t in a.tags {
                if !item.tags.0.iter().any(|x| x.eq_ignore_ascii_case(&t)) {
                    item.tags.0.push(t);
                }
            }
            sqlx::query(
                "UPDATE inbox_items SET status = 'pruefen', step = '', warning = '', name = ?, tags = ?, metadata = ?, summary = ?,
                 model = ?, updated_at = ? WHERE id = ? AND status = 'analyse'",
            )
            .bind(&item.name)
            .bind(&item.tags)
            .bind(sqlx::types::Json(&a.metadata))
            .bind(&a.summary)
            .bind(&a.model)
            .bind(now())
            .bind(&id)
            .execute(&pool)
            .await
            .map(|_| ())
        }
        Ok((_, Err(e))) => {
            warn(
                &pool,
                &id,
                &format!("{e} – Name und Tags bitte selbst eintragen."),
            )
            .await
        }
        Err(e) => {
            warn(
                &pool,
                &id,
                &format!("KI-Analyse nicht möglich: {}", e.public_message()),
            )
            .await
        }
    };
    if let Err(e) = outcome {
        tracing::warn!(inbox = %id, "Prüfliste: Ergebnis der KI nicht gespeichert: {e}");
    }
}

async fn warn(pool: &SqlitePool, id: &str, warning: &str) -> Result<(), sqlx::Error> {
    tracing::warn!(inbox = %id, "{warning}");
    sqlx::query("UPDATE inbox_items SET status = 'pruefen', step = '', warning = ?, updated_at = ? WHERE id = ? AND status = 'analyse'")
        .bind(warning)
        .bind(now())
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
}

/// Freigabe: Änderungen übernehmen und als Dokument ablegen – mit Tags (fehlende werden angelegt)
/// und Metadaten als Eigenschaften. Gibt die ID des neuen Dokuments zurück.
pub async fn approve(
    pool: &SqlitePool,
    queue: &OcrQueue,
    org: &str,
    id: &str,
    user: &str,
    e: Edit,
) -> AppResult<String> {
    let item = edit(pool, org, id, e).await?;
    let org = item.organization_id.clone();
    set_status(pool, id, InboxStatus::Import, "Wird abgelegt …", "").await?;
    let result = async {
        let pdf = file(pool, &org, id, 0).await?;
        let new = NewDocument {
            organization_id: &org,
            user_id: Some(user),
            source: DocumentSource::Web,
            file_name: &format!("{}.pdf", item.name),
            data: pdf,
        };
        let doc = documents::ingest(pool, queue, new)
            .await?
            .document()
            .clone();
        let existing = tags::list(pool, &org).await?;
        for (i, name) in item.tags.0.iter().enumerate() {
            let tag = match tags::by_name(pool, &org, name).await? {
                Some(t) => t,
                None => {
                    let color = tags::PALETTE[(existing.len() + i) % tags::PALETTE.len()];
                    tags::create(pool, &org, name, color, "").await?
                }
            };
            tags::add_to_document(pool, &org, &doc.id, &tag.id, Some(user)).await?;
        }
        let warnings =
            properties::apply_scan_metadata(pool, &org, &doc.id, &item.metadata.0, &item.summary)
                .await?;
        documents::log(
            pool,
            &doc.id,
            "reviewed",
            Some(user),
            serde_json::json!({ "model": item.model, "pages": item.page_count }),
        )
        .await?;
        Ok::<_, AppError>((doc.id, warnings))
    }
    .await;

    match result {
        Ok((document_id, warnings)) => {
            let mut tx = pool.begin().await?;
            sqlx::query("DELETE FROM inbox_files WHERE inbox_id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                "UPDATE inbox_items SET status = 'fertig', step = '', error = '', warning = ?, document_id = ?, updated_at = ? WHERE id = ?",
            )
            .bind(warnings.join("; "))
            .bind(&document_id)
            .bind(now())
            .bind(id)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(document_id)
        }
        Err(e) => {
            set_status(pool, id, InboxStatus::Fehler, "", &e.public_message()).await?;
            Err(e)
        }
    }
}

/// Verwerfen: Eintrag samt PDF und Seiten löschen (abgelegte Dokumente bleiben).
pub async fn delete(pool: &SqlitePool, org: &str, id: &str) -> AppResult<()> {
    let item = get(pool, org, id).await?;
    if matches!(item.status, InboxStatus::Analyse | InboxStatus::Import) {
        return Err(AppError::Conflict(
            "Das Dokument wird gerade bearbeitet – kurz warten.".into(),
        ));
    }
    sqlx::query("DELETE FROM inbox_items WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Nach einem Neustart: unterbrochene Analysen wieder prüfbar machen, unterbrochene Ablagen als Fehler.
pub async fn recover(pool: &SqlitePool) -> AppResult<u64> {
    let a = sqlx::query(
        "UPDATE inbox_items SET status = 'pruefen', step = '', warning = 'Durch Neustart unterbrochen – neu analysieren.' WHERE status = 'analyse'",
    )
    .execute(pool)
    .await?
    .rows_affected();
    let b = sqlx::query("UPDATE inbox_items SET status = 'fehler', step = '', error = 'Durch Neustart unterbrochen.' WHERE status = 'import'")
        .execute(pool)
        .await?
        .rows_affected();
    Ok(a + b)
}

/// Einträge löschen, die länger als `days` Tage liegen (laufende Arbeiten bleiben).
pub async fn purge(pool: &SqlitePool, days: u32) -> AppResult<u64> {
    Ok(sqlx::query(
        "DELETE FROM inbox_items WHERE created_at < ? AND status NOT IN ('analyse', 'import')",
    )
    .bind(now() - days as i64 * 86_400)
    .execute(pool)
    .await?
    .rows_affected())
}

#[cfg(test)]
mod tests {
    #[test]
    fn names() {
        assert_eq!(super::clean_name("  Rechnung: Juni.pdf "), "Rechnung- Juni");
        assert_eq!(super::clean_name("a/b\\c"), "a-b-c");
        assert!(super::clean_name("   ").starts_with("Scan 20"));
    }
}

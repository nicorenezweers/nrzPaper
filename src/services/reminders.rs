//! Erinnerungen an Dokumente („Erinnere mich in 10 Tagen“). Sie gehören einer Person: jede
//! sieht nur ihre eigenen. Fällige Erinnerungen erscheinen in der Seitenleiste und oben in
//! der Zeitleiste, bis sie erledigt oder verschoben sind.

use crate::db::models::{new_id, now};
use crate::error::{AppError, AppResult};
use crate::services::documents;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub organization_id: String,
    pub document_id: String,
    pub document_name: String,
    pub due_at: i64,
    pub note: String,
    pub done_at: Option<i64>,
    pub created_at: i64,
}

/// Abfrage mit Dokumentname plus Bedingung – als fester String (kein dynamisches SQL).
macro_rules! select {
    ($rest:literal) => {
        concat!(
            "SELECT r.id, r.organization_id, r.document_id, d.name AS document_name, r.due_at, r.note, r.done_at, r.created_at
             FROM reminders r JOIN documents d ON d.id = r.document_id ",
            $rest
        )
    };
}

/// Höchstens zehn Jahre im Voraus, nicht in der Vergangenheit (eine Minute Spielraum).
fn check_due(due_at: i64) -> AppResult<()> {
    let n = now();
    if due_at < n - 60 || due_at > n + 3650 * 86_400 {
        return Err(AppError::bad(
            "Der Zeitpunkt muss in der Zukunft liegen (höchstens 10 Jahre).",
        ));
    }
    Ok(())
}

fn check_note(note: &str) -> AppResult<String> {
    let note = note.trim();
    if note.chars().count() > 500 {
        return Err(AppError::bad("Notiz höchstens 500 Zeichen."));
    }
    Ok(note.to_string())
}

pub async fn create(
    pool: &SqlitePool,
    org: &str,
    user: &str,
    document: &str,
    due_at: i64,
    note: &str,
) -> AppResult<Reminder> {
    let doc = documents::get(pool, org, document).await?;
    if doc.in_trash() {
        return Err(AppError::bad("Dieses Dokument liegt im Papierkorb."));
    }
    check_due(due_at)?;
    let id = new_id();
    sqlx::query(
        "INSERT INTO reminders (id, organization_id, document_id, user_id, due_at, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(org)
    .bind(document)
    .bind(user)
    .bind(due_at)
    .bind(check_note(note)?)
    .bind(now())
    .execute(pool)
    .await?;
    get(pool, user, &id).await
}

pub async fn get(pool: &SqlitePool, user: &str, id: &str) -> AppResult<Reminder> {
    sqlx::query_as(select!("WHERE r.id = ? AND r.user_id = ?"))
        .bind(id)
        .bind(user)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

/// Eigene Erinnerungen zu einem Dokument (offene zuerst).
pub async fn for_document(pool: &SqlitePool, user: &str, document: &str) -> AppResult<Vec<Reminder>> {
    Ok(sqlx::query_as(select!(
        "WHERE r.user_id = ? AND r.document_id = ? ORDER BY r.done_at IS NOT NULL, r.due_at"
    ))
    .bind(user)
    .bind(document)
    .fetch_all(pool)
    .await?)
}

/// Offene Erinnerungen einer Organisation (Dokumente im Papierkorb ausgenommen), nach Fälligkeit.
pub async fn open(pool: &SqlitePool, user: &str, org: &str) -> AppResult<Vec<Reminder>> {
    Ok(sqlx::query_as(select!(
        "WHERE r.user_id = ? AND r.organization_id = ? AND r.done_at IS NULL AND d.deleted_at IS NULL ORDER BY r.due_at"
    ))
    .bind(user)
    .bind(org)
    .fetch_all(pool)
    .await?)
}

/// Anzahl fälliger offener Erinnerungen (für die Seitenleiste).
pub async fn due_count(pool: &SqlitePool, user: &str, org: &str) -> AppResult<i64> {
    Ok(sqlx::query_scalar(
        "SELECT count(*) FROM reminders r JOIN documents d ON d.id = r.document_id
         WHERE r.user_id = ? AND r.organization_id = ? AND r.done_at IS NULL AND r.due_at <= ? AND d.deleted_at IS NULL",
    )
    .bind(user)
    .bind(org)
    .bind(now())
    .fetch_one(pool)
    .await?)
}

/// Erledigt/offen setzen, verschieben oder Notiz ändern.
pub async fn update(
    pool: &SqlitePool,
    user: &str,
    id: &str,
    done: Option<bool>,
    due_at: Option<i64>,
    note: Option<&str>,
) -> AppResult<Reminder> {
    let r = get(pool, user, id).await?;
    let done_at = match done {
        Some(true) => Some(r.done_at.unwrap_or_else(now)),
        Some(false) => None,
        None => r.done_at,
    };
    let due = match due_at {
        Some(d) => {
            check_due(d)?;
            d
        }
        None => r.due_at,
    };
    let note = match note {
        Some(n) => check_note(n)?,
        None => r.note,
    };
    sqlx::query("UPDATE reminders SET done_at = ?, due_at = ?, note = ? WHERE id = ? AND user_id = ?")
        .bind(done_at)
        .bind(due)
        .bind(note)
        .bind(id)
        .bind(user)
        .execute(pool)
        .await?;
    get(pool, user, id).await
}

pub async fn delete(pool: &SqlitePool, user: &str, id: &str) -> AppResult<()> {
    let n = sqlx::query("DELETE FROM reminders WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

//! Freigabelinks: ein Dokument ohne Anmeldung zeigen. Der Token (`sh_…`) wird nur einmal
//! angezeigt; gespeichert ist sein SHA-256. Ein Link gilt bis zum Ablauf oder Widerruf und
//! nie für Dokumente im Papierkorb.

use crate::db::models::{new_id, now, Document, ShareLink};
use crate::error::{AppError, AppResult};
use crate::services::documents;
use base64::Engine;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

pub const PREFIX: &str = "sh_";

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// Neuer Link; `expires_in_days` = None heisst unbefristet. Zurück: Datensatz und Token.
pub async fn create(
    pool: &SqlitePool,
    org: &str,
    document_id: &str,
    user: Option<&str>,
    expires_in_days: Option<u32>,
) -> AppResult<(ShareLink, String)> {
    let doc = documents::get(pool, org, document_id).await?;
    if doc.in_trash() {
        return Err(AppError::bad(
            "Dokumente im Papierkorb lassen sich nicht freigeben.",
        ));
    }
    if expires_in_days.is_some_and(|d| d == 0 || d > 3650) {
        return Err(AppError::bad("Ablauf: 1 bis 3650 Tage."));
    }
    let token = format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(rand::random::<[u8; 24]>())
    );
    let id = new_id();
    let ts = now();
    sqlx::query("INSERT INTO share_links (id, document_id, token_hash, created_by, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(document_id)
        .bind(hash(&token))
        .bind(user)
        .bind(ts)
        .bind(expires_in_days.map(|d| ts + d as i64 * 86_400))
        .execute(pool)
        .await?;
    documents::log(pool, document_id, "shared", user, serde_json::json!({})).await?;
    let link = sqlx::query_as("SELECT * FROM share_links WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await?;
    Ok((link, token))
}

/// Links eines Dokuments, neueste zuerst.
pub async fn list(pool: &SqlitePool, org: &str, document_id: &str) -> AppResult<Vec<ShareLink>> {
    documents::get(pool, org, document_id).await?;
    Ok(sqlx::query_as(
        "SELECT * FROM share_links WHERE document_id = ? ORDER BY created_at DESC, id DESC",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?)
}

pub async fn revoke(
    pool: &SqlitePool,
    org: &str,
    document_id: &str,
    id: &str,
    user: Option<&str>,
) -> AppResult<()> {
    documents::get(pool, org, document_id).await?;
    let done = sqlx::query("UPDATE share_links SET revoked_at = ? WHERE id = ? AND document_id = ? AND revoked_at IS NULL")
        .bind(now())
        .bind(id)
        .bind(document_id)
        .execute(pool)
        .await?
        .rows_affected();
    if done == 0 {
        return Err(AppError::NotFound);
    }
    documents::log(
        pool,
        document_id,
        "share_revoked",
        user,
        serde_json::json!({}),
    )
    .await
}

/// Gültigen Link samt Dokument finden – sonst 404 (abgelaufen, widerrufen, Papierkorb,
/// Organisation gelöscht). `count_view` zählt einen Seitenaufruf.
pub async fn resolve(
    pool: &SqlitePool,
    token: &str,
    count_view: bool,
) -> AppResult<(ShareLink, Document)> {
    if !token.starts_with(PREFIX) || token.len() > 64 {
        return Err(AppError::NotFound);
    }
    let link: ShareLink = sqlx::query_as("SELECT * FROM share_links WHERE token_hash = ?")
        .bind(hash(token))
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    if !link.is_active(now()) {
        return Err(AppError::NotFound);
    }
    let doc: Document = sqlx::query_as(
        "SELECT d.* FROM documents d JOIN organizations o ON o.id = d.organization_id
         WHERE d.id = ? AND d.deleted_at IS NULL AND o.deleted_at IS NULL",
    )
    .bind(&link.document_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;
    if count_view {
        sqlx::query("UPDATE share_links SET view_count = view_count + 1 WHERE id = ?")
            .bind(&link.id)
            .execute(pool)
            .await?;
    }
    Ok((link, doc))
}

//! API-Keys: `np_` + 40 zufällige Zeichen. Gezeigt wird der Key nur einmal beim Anlegen;
//! gespeichert werden der SHA-256 und die ersten Zeichen zur Wiedererkennung.

use crate::db::models::{new_id, now, ApiKey, Permission};
use crate::error::{AppError, AppResult};
use base64::Engine;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

pub const PREFIX: &str = "np_";

pub fn hash_key(key: &str) -> String {
    hex::encode(Sha256::digest(key.as_bytes()))
}

fn generate() -> String {
    let random =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(rand::random::<[u8; 30]>());
    format!("{PREFIX}{random}")
}

/// Neuer Key; zurück kommt der Datensatz und der Key im Klartext (nur dieses eine Mal).
pub async fn create(
    pool: &SqlitePool,
    user_id: &str,
    name: &str,
    permissions: &[Permission],
    expires_at: Option<i64>,
) -> AppResult<(ApiKey, String)> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 64 {
        return Err(AppError::bad("Name des API-Keys: 1–64 Zeichen."));
    }
    if permissions.is_empty() {
        return Err(AppError::bad("Mindestens eine Berechtigung wählen."));
    }
    let key = generate();
    let id = new_id();
    sqlx::query("INSERT INTO api_keys (id, user_id, name, prefix, key_hash, permissions, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(user_id)
        .bind(name)
        .bind(&key[..PREFIX.len() + 6])
        .bind(hash_key(&key))
        .bind(sqlx::types::Json(permissions))
        .bind(now())
        .bind(expires_at)
        .execute(pool)
        .await?;
    let record = sqlx::query_as("SELECT * FROM api_keys WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await?;
    Ok((record, key))
}

/// Key prüfen (bekannt, nicht abgelaufen) und „zuletzt benutzt“ setzen.
pub async fn lookup(pool: &SqlitePool, key: &str) -> AppResult<Option<ApiKey>> {
    if !key.starts_with(PREFIX) {
        return Ok(None);
    }
    let ts = now();
    let record: Option<ApiKey> = sqlx::query_as(
        "SELECT * FROM api_keys WHERE key_hash = ? AND (expires_at IS NULL OR expires_at > ?)",
    )
    .bind(hash_key(key))
    .bind(ts)
    .fetch_optional(pool)
    .await?;
    if let Some(k) = &record {
        sqlx::query("UPDATE api_keys SET last_used_at = ? WHERE id = ?")
            .bind(ts)
            .bind(&k.id)
            .execute(pool)
            .await?;
    }
    Ok(record)
}

pub async fn list(pool: &SqlitePool, user_id: &str) -> AppResult<Vec<ApiKey>> {
    Ok(
        sqlx::query_as("SELECT * FROM api_keys WHERE user_id = ? ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn delete(pool: &SqlitePool, user_id: &str, id: &str) -> AppResult<()> {
    let n = sqlx::query("DELETE FROM api_keys WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

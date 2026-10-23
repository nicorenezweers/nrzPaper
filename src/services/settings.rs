//! Instanzweite Einstellungen in `app_settings` (JSON-Werte).

use crate::db::models::now;
use crate::error::AppResult;
use base64::Engine;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::SqlitePool;

pub const REGISTRATION_OPEN: &str = "registration_open";
const COOKIE_SECRET: &str = "cookie_secret";

pub async fn get<T: DeserializeOwned>(pool: &SqlitePool, key: &str) -> AppResult<Option<T>> {
    let raw: Option<String> = sqlx::query_scalar("SELECT value FROM app_settings WHERE key = ?").bind(key).fetch_optional(pool).await?;
    Ok(raw.and_then(|v| serde_json::from_str(&v).ok()))
}

pub async fn set<T: Serialize>(pool: &SqlitePool, key: &str, value: &T) -> AppResult<()> {
    let json = serde_json::to_string(value).expect("serialisierbar");
    sqlx::query(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(json)
    .bind(now())
    .execute(pool)
    .await?;
    Ok(())
}

/// Schlüssel für Rockets private Cookies. Liegt verschlüsselt in der Datenbank, damit
/// Sitzungen einen Neustart überstehen, ohne dass ein weiteres Geheimnis konfiguriert werden muss.
pub async fn cookie_secret(pool: &SqlitePool) -> AppResult<String> {
    if let Some(secret) = get::<String>(pool, COOKIE_SECRET).await? {
        return Ok(secret);
    }
    let secret = base64::engine::general_purpose::STANDARD.encode(rand::random::<[u8; 32]>());
    set(pool, COOKIE_SECRET, &secret).await?;
    Ok(secret)
}

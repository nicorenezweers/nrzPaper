//! Sitzungen: Der Browser bekommt ein zufälliges Token in einem privaten (von Rocket
//! verschlüsselten) Cookie; in der Datenbank liegt nur dessen SHA-256. Sitzungen laufen
//! 30 Tage und verlängern sich bei Benutzung.

use crate::db::models::{now, User};
use crate::error::AppResult;
use base64::Engine;
use rocket::http::{Cookie, CookieJar, SameSite};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

pub const COOKIE: &str = "nrzpaper_session";
pub const LIFETIME_SECS: i64 = 30 * 86_400;
/// Seltener als einmal pro Stunde wird die Sitzung nicht verlängert (spart Schreibzugriffe).
const REFRESH_AFTER_SECS: i64 = 3_600;

pub fn new_token() -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>())
}

pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub async fn create(pool: &SqlitePool, user_id: &str, user_agent: &str) -> AppResult<String> {
    let token = new_token();
    let ts = now();
    sqlx::query("INSERT INTO sessions (token_hash, user_id, created_at, expires_at, last_seen_at, user_agent) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(hash_token(&token))
        .bind(user_id)
        .bind(ts)
        .bind(ts + LIFETIME_SECS)
        .bind(ts)
        .bind(user_agent.chars().take(300).collect::<String>())
        .execute(pool)
        .await?;
    Ok(token)
}

/// Benutzer zur Sitzung – oder `None`, wenn sie unbekannt oder abgelaufen ist.
pub async fn lookup(pool: &SqlitePool, token: &str) -> AppResult<Option<User>> {
    let hash = hash_token(token);
    let ts = now();
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT last_seen_at FROM sessions WHERE token_hash = ? AND expires_at > ?")
            .bind(&hash)
            .bind(ts)
            .fetch_optional(pool)
            .await?;
    let Some((last_seen,)) = row else {
        return Ok(None);
    };
    if ts - last_seen > REFRESH_AFTER_SECS {
        sqlx::query("UPDATE sessions SET last_seen_at = ?, expires_at = ? WHERE token_hash = ?")
            .bind(ts)
            .bind(ts + LIFETIME_SECS)
            .bind(&hash)
            .execute(pool)
            .await?;
    }
    Ok(sqlx::query_as(
        "SELECT u.* FROM users u JOIN sessions s ON s.user_id = u.id WHERE s.token_hash = ?",
    )
    .bind(&hash)
    .fetch_optional(pool)
    .await?)
}

pub async fn destroy(pool: &SqlitePool, token: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
        .bind(hash_token(token))
        .execute(pool)
        .await?;
    Ok(())
}

/// Alle Sitzungen eines Benutzers beenden (z. B. nach Passwortänderung).
pub async fn destroy_all(pool: &SqlitePool, user_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn purge_expired(pool: &SqlitePool) -> AppResult<u64> {
    Ok(sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
        .bind(now())
        .execute(pool)
        .await?
        .rows_affected())
}

/// Cookie setzen: privat (verschlüsselt + signiert), HttpOnly, SameSite=Lax.
/// SameSite=Lax schickt das Cookie bei fremden POST-Anfragen nicht mit – Grundschutz gegen CSRF.
pub fn set_cookie(cookies: &CookieJar<'_>, token: String, secure: bool) {
    let cookie = Cookie::build((COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(rocket::time::Duration::seconds(LIFETIME_SECS));
    cookies.add_private(cookie);
}

pub fn token_from(cookies: &CookieJar<'_>) -> Option<String> {
    cookies.get_private(COOKIE).map(|c| c.value().to_string())
}

pub fn clear_cookie(cookies: &CookieJar<'_>) {
    cookies.remove_private(Cookie::build(COOKIE).path("/"));
}

//! Benutzerkonten.

use crate::auth::password;
use crate::config::Config;
use crate::db::models::{new_id, now, User};
use crate::error::{AppError, AppResult};
use crate::services::settings;
use sqlx::SqlitePool;

pub async fn count(pool: &SqlitePool) -> AppResult<i64> {
    Ok(sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(pool)
        .await?)
}

pub async fn by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<User>> {
    Ok(sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn by_username(pool: &SqlitePool, username: &str) -> AppResult<Option<User>> {
    Ok(sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(username.trim())
        .fetch_optional(pool)
        .await?)
}

/// Darf sich gerade jemand selbst registrieren? Das erste Konto immer, danach nur,
/// wenn die Registrierung offen ist (Einstellung in der Oberfläche, sonst Konfiguration).
pub async fn registration_allowed(pool: &SqlitePool, config: &Config) -> AppResult<bool> {
    if count(pool).await? == 0 {
        return Ok(true);
    }
    Ok(settings::get::<bool>(pool, settings::REGISTRATION_OPEN)
        .await?
        .unwrap_or(config.registration_open))
}

pub fn validate_username(name: &str) -> AppResult<String> {
    let name = name.trim();
    let ok = (3..=32).contains(&name.chars().count())
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || "._-".contains(c));
    if !ok {
        return Err(AppError::bad(
            "Benutzername: 3–32 Zeichen, nur Buchstaben, Zahlen, Punkt, Minus und Unterstrich.",
        ));
    }
    Ok(name.to_string())
}

/// Legt ein Konto an. Das allererste Konto wird Administrator der Instanz.
pub async fn create(
    pool: &SqlitePool,
    username: &str,
    password_plain: &str,
    display_name: &str,
) -> AppResult<User> {
    let username = validate_username(username)?;
    password::check_strength(password_plain)?;
    let hash = password::hash(password_plain.to_string()).await?;
    let first = count(pool).await? == 0;
    let id = new_id();
    let ts = now();
    sqlx::query(
        "INSERT INTO users (id, username, display_name, password_hash, is_admin, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&username)
    .bind(display_name.trim())
    .bind(&hash)
    .bind(first)
    .bind(ts)
    .bind(ts)
    .execute(pool)
    .await
    .map_err(|e| AppError::unique(e, "Dieser Benutzername ist schon vergeben."))?;
    Ok(by_id(pool, &id).await?.expect("gerade angelegt"))
}

/// Anmeldedaten prüfen. Auch bei unbekanntem Namen wird ein Hash geprüft, damit die
/// Antwortzeit nicht verrät, ob es das Konto gibt.
pub async fn authenticate(
    pool: &SqlitePool,
    username: &str,
    password_plain: &str,
) -> AppResult<Option<User>> {
    let user = by_username(pool, username).await?;
    let hash = user
        .as_ref()
        .map(|u| u.password_hash.clone())
        .unwrap_or_else(|| password::DUMMY_HASH.clone());
    let ok = password::verify(password_plain.to_string(), hash).await;
    Ok(user.filter(|_| ok))
}

/// Passwort ändern (altes Passwort nötig). Danach sind alle Sitzungen des Kontos ungültig;
/// wer gerade ändert, bekommt vom Aufrufer eine neue.
pub async fn change_password(
    pool: &SqlitePool,
    user: &User,
    current: &str,
    new: &str,
    confirm: &str,
) -> AppResult<()> {
    if !password::verify(current.to_string(), user.password_hash.clone()).await {
        return Err(AppError::bad("Das aktuelle Passwort stimmt nicht."));
    }
    if new != confirm {
        return Err(AppError::bad("Die neuen Passwörter stimmen nicht überein."));
    }
    password::check_strength(new)?;
    let hash = password::hash(new.to_string()).await?;
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(hash)
        .bind(now())
        .bind(&user.id)
        .execute(pool)
        .await?;
    crate::auth::session::destroy_all(pool, &user.id).await
}

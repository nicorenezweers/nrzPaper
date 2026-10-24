//! Scannen: Scanner-Einstellungen je Benutzer und die laufende Scan-Sitzung je Benutzer und
//! Organisation (Name, vorgegebene Tags, Seiten in Reihenfolge). "Zur Prüfung" macht daraus ein
//! PDF und einen Eintrag der Prüfliste.

use crate::ai::AiConfig;
use crate::config::Config;
use crate::db::models::{new_id, now, InboxItem, ScanSettings};
use crate::error::{AppError, AppResult};
use crate::services::inbox;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashSet;

/// Einstellungen des Benutzers; ohne eigene gilt der voreingestellte Scanner.
pub async fn settings(pool: &SqlitePool, config: &Config, user: &str) -> AppResult<ScanSettings> {
    let row: Option<ScanSettings> = sqlx::query_as("SELECT * FROM scan_settings WHERE user_id = ?")
        .bind(user)
        .fetch_optional(pool)
        .await?;
    Ok(row.unwrap_or_else(|| ScanSettings {
        user_id: user.to_string(),
        scanner_url: config.scanner_url.clone(),
        color: true,
        dpi: 300,
        updated_at: 0,
    }))
}

/// Scanner-Adresse: "192.168.0.70" → "http://192.168.0.70", ohne Schrägstrich am Ende.
pub fn clean_scanner_url(url: &str) -> AppResult<String> {
    let url = url.trim();
    if url.is_empty() {
        return Ok(String::new());
    }
    // Erst das Schema ergänzen und prüfen, dann den Schrägstrich am Ende entfernen –
    // sonst würde aus "http://" der Host "http".
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{url}")
    };
    let parsed =
        reqwest::Url::parse(&url).map_err(|_| AppError::bad("Ungültige Scanner-Adresse."))?;
    if parsed.host_str().is_none_or(str::is_empty) {
        return Err(AppError::bad("Ungültige Scanner-Adresse."));
    }
    Ok(url.trim_end_matches('/').to_string())
}

pub async fn save_settings(
    pool: &SqlitePool,
    user: &str,
    scanner_url: &str,
    color: bool,
    dpi: i64,
) -> AppResult<()> {
    let url = clean_scanner_url(scanner_url)?;
    let dpi = if [150, 300, 600].contains(&dpi) {
        dpi
    } else {
        300
    };
    sqlx::query(
        "INSERT INTO scan_settings (user_id, scanner_url, color, dpi, updated_at) VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (user_id) DO UPDATE SET scanner_url = excluded.scanner_url, color = excluded.color,
         dpi = excluded.dpi, updated_at = excluded.updated_at",
    )
    .bind(user)
    .bind(url)
    .bind(color)
    .bind(dpi)
    .bind(now())
    .execute(pool)
    .await?;
    Ok(())
}

/// Die laufende Sitzung für die Oberfläche.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub name: String,
    pub hint_tags: Vec<String>,
    /// Seiten-IDs in Reihenfolge
    pub pages: Vec<String>,
}

pub async fn session(pool: &SqlitePool, user: &str, org: &str) -> AppResult<Session> {
    let row: Option<(String, sqlx::types::Json<Vec<String>>)> = sqlx::query_as(
        "SELECT name, hint_tags FROM scan_sessions WHERE user_id = ? AND organization_id = ?",
    )
    .bind(user)
    .bind(org)
    .fetch_optional(pool)
    .await?;
    let pages = sqlx::query_scalar("SELECT id FROM scan_pages WHERE user_id = ? AND organization_id = ? ORDER BY position, created_at")
        .bind(user)
        .bind(org)
        .fetch_all(pool)
        .await?;
    let (name, hint_tags) = row.map(|(n, h)| (n, h.0)).unwrap_or_default();
    Ok(Session {
        name,
        hint_tags,
        pages,
    })
}

async fn write_session(
    pool: &SqlitePool,
    user: &str,
    org: &str,
    name: &str,
    hint_tags: &[String],
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO scan_sessions (user_id, organization_id, name, hint_tags, updated_at) VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (user_id, organization_id) DO UPDATE SET name = excluded.name, hint_tags = excluded.hint_tags,
         updated_at = excluded.updated_at",
    )
    .bind(user)
    .bind(org)
    .bind(name.chars().take(200).collect::<String>())
    .bind(sqlx::types::Json(hint_tags))
    .bind(now())
    .execute(pool)
    .await?;
    Ok(())
}

/// Name, vorgegebene Tags und/oder Reihenfolge ändern. Die Reihenfolge muss genau die
/// vorhandenen Seiten enthalten – umsortieren, nichts hinzufügen oder verlieren.
pub async fn update(
    pool: &SqlitePool,
    user: &str,
    org: &str,
    name: Option<&str>,
    hint_tags: Option<Vec<String>>,
    order: Option<Vec<String>>,
) -> AppResult<Session> {
    let current = session(pool, user, org).await?;
    if let Some(order) = order {
        let a: HashSet<_> = order.iter().collect();
        let b: HashSet<_> = current.pages.iter().collect();
        if a != b || order.len() != current.pages.len() {
            return Err(AppError::bad("Seitenreihenfolge passt nicht zur Sitzung."));
        }
        let mut tx = pool.begin().await?;
        for (i, id) in order.iter().enumerate() {
            sqlx::query("UPDATE scan_pages SET position = ? WHERE id = ? AND user_id = ?")
                .bind(i as i64)
                .bind(id)
                .bind(user)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
    }
    if name.is_some() || hint_tags.is_some() {
        let name = name.map(str::to_string).unwrap_or(current.name);
        let hints: Vec<String> = hint_tags
            .unwrap_or(current.hint_tags)
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        write_session(pool, user, org, &name, &hints).await?;
    }
    session(pool, user, org).await
}

/// Seite (JPEG) hinten anhängen.
pub async fn add_page(
    pool: &SqlitePool,
    user: &str,
    org: &str,
    jpeg: Vec<u8>,
) -> AppResult<Session> {
    if !crate::scan::is_jpeg(&jpeg) {
        return Err(AppError::bad(
            "Nur JPG-Bilder als Seiten – PDFs bitte über „PDF prüfen lassen“ importieren.",
        ));
    }
    sqlx::query(
        "INSERT INTO scan_pages (id, user_id, organization_id, position, data, created_at)
         VALUES (?, ?, ?, (SELECT coalesce(max(position), -1) + 1 FROM scan_pages WHERE user_id = ? AND organization_id = ?), ?, ?)",
    )
    .bind(new_id())
    .bind(user)
    .bind(org)
    .bind(user)
    .bind(org)
    .bind(jpeg)
    .bind(now())
    .execute(pool)
    .await?;
    session(pool, user, org).await
}

pub async fn page(pool: &SqlitePool, user: &str, org: &str, id: &str) -> AppResult<Vec<u8>> {
    sqlx::query_scalar(
        "SELECT data FROM scan_pages WHERE id = ? AND user_id = ? AND organization_id = ?",
    )
    .bind(id)
    .bind(user)
    .bind(org)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn delete_page(pool: &SqlitePool, user: &str, org: &str, id: &str) -> AppResult<Session> {
    sqlx::query("DELETE FROM scan_pages WHERE id = ? AND user_id = ? AND organization_id = ?")
        .bind(id)
        .bind(user)
        .bind(org)
        .execute(pool)
        .await?;
    session(pool, user, org).await
}

/// Alle Seiten verwerfen, Name und Tag-Vorgaben leeren.
pub async fn clear(pool: &SqlitePool, user: &str, org: &str) -> AppResult<Session> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM scan_pages WHERE user_id = ? AND organization_id = ?")
        .bind(user)
        .bind(org)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM scan_sessions WHERE user_id = ? AND organization_id = ?")
        .bind(user)
        .bind(org)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    session(pool, user, org).await
}

/// Seiten zu einem PDF zusammenfügen und zur Prüfung geben (die KI liest im Hintergrund).
pub async fn finish(
    pool: &SqlitePool,
    ai: &AiConfig,
    user: &str,
    org: &str,
) -> AppResult<InboxItem> {
    let s = session(pool, user, org).await?;
    if s.pages.is_empty() {
        return Err(AppError::bad("Keine Seiten gescannt."));
    }
    let mut jpegs = Vec::with_capacity(s.pages.len());
    for id in &s.pages {
        jpegs.push(page(pool, user, org, id).await?);
    }
    let pdf = crate::scan::make_pdf(&jpegs).map_err(AppError::bad)?;
    let item = inbox::create(
        pool,
        ai,
        org,
        Some(user),
        &s.name,
        s.name.trim().is_empty(),
        s.hint_tags.clone(),
        pdf,
        jpegs,
    )
    .await?;
    clear(pool, user, org).await?;
    Ok(item)
}

#[cfg(test)]
mod tests {
    #[test]
    fn scanner_urls() {
        let c = |u: &str| super::clean_scanner_url(u).ok();
        assert_eq!(c("192.168.0.70").as_deref(), Some("http://192.168.0.70"));
        assert_eq!(
            c("http://192.168.0.70/").as_deref(),
            Some("http://192.168.0.70")
        );
        assert_eq!(
            c("https://scanner.local:8443").as_deref(),
            Some("https://scanner.local:8443")
        );
        assert_eq!(c("").as_deref(), Some(""));
        assert_eq!(c("http://"), None);
        assert_eq!(c("http:// /"), None);
    }
}

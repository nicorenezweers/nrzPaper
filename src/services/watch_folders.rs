//! Ordner-Eingang: Ordner auf dem Server (z. B. der Scan-Ordner eines Multifunktionsgeräts auf
//! dem NAS, in Docker als Volume eingebunden), deren PDFs und Bilder zur Prüfung in die Inbox
//! kommen – mit vorgegebenen Tags und den Vorschlägen der KI. Danach wird die Datei gelöscht
//! oder in den Unterordner `nrzpaper-importiert/` verschoben; was sich nicht übernehmen lässt,
//! kommt nach `nrzpaper-fehler/`. Nur Administratoren der Instanz richten Ordner ein, weil sie
//! Pfade auf dem Server betreffen.

use crate::ai::AiConfig;
use crate::db::models::{new_id, now};
use crate::error::{AppError, AppResult};
use crate::services::inbox;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const DONE_DIR: &str = "nrzpaper-importiert";
pub const FAILED_DIR: &str = "nrzpaper-fehler";
/// So lange darf sich eine Datei nicht mehr ändern, bevor sie übernommen wird.
const SETTLE: Duration = Duration::from_secs(3);
/// Grösser wird nichts übernommen (wie beim Upload).
const MAX_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WatchFolder {
    pub id: String,
    pub organization_id: String,
    pub created_by: Option<String>,
    pub name: String,
    pub path: String,
    pub tags: sqlx::types::Json<Vec<String>>,
    pub delete_after: bool,
    pub enabled: bool,
    pub last_check_at: Option<i64>,
    pub last_error: String,
    pub imported_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub organization_id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub delete_after: bool,
    #[serde(default = "yes")]
    pub enabled: bool,
}

fn yes() -> bool {
    true
}

async fn validate(i: &Input) -> AppResult<(String, String, Vec<String>)> {
    let name = i.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::bad("Name des Ordners: 1–100 Zeichen."));
    }
    let path = i.path.trim().trim_end_matches(['/', '\\']);
    let p = Path::new(path);
    if !p.is_absolute() {
        return Err(AppError::bad("Bitte einen absoluten Pfad angeben, z. B. /scans."));
    }
    match tokio::fs::metadata(p).await {
        Ok(m) if m.is_dir() => {}
        Ok(_) => return Err(AppError::bad("Der Pfad ist kein Ordner.")),
        Err(_) => {
            return Err(AppError::bad(
                "Ordner nicht gefunden – in Docker muss er als Volume eingebunden sein.",
            ))
        }
    }
    let tags: Vec<String> = i
        .tags
        .iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(20)
        .collect();
    Ok((name.to_string(), path.to_string(), tags))
}

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<WatchFolder>> {
    Ok(sqlx::query_as("SELECT * FROM watch_folders ORDER BY name COLLATE NOCASE")
        .fetch_all(pool)
        .await?)
}

/// Ordner, die in eine Organisation liefern (für den Filter in der Inbox).
pub async fn for_org(pool: &SqlitePool, org: &str) -> AppResult<Vec<WatchFolder>> {
    Ok(sqlx::query_as("SELECT * FROM watch_folders WHERE organization_id = ? ORDER BY name COLLATE NOCASE")
        .bind(org)
        .fetch_all(pool)
        .await?)
}

pub async fn get(pool: &SqlitePool, id: &str) -> AppResult<WatchFolder> {
    sqlx::query_as("SELECT * FROM watch_folders WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create(pool: &SqlitePool, user: &str, i: &Input) -> AppResult<WatchFolder> {
    let (name, path, tags) = validate(i).await?;
    let id = new_id();
    let ts = now();
    sqlx::query(
        "INSERT INTO watch_folders (id, organization_id, created_by, name, path, tags, delete_after, enabled, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&i.organization_id)
    .bind(user)
    .bind(name)
    .bind(path)
    .bind(sqlx::types::Json(tags))
    .bind(i.delete_after)
    .bind(i.enabled)
    .bind(ts)
    .bind(ts)
    .execute(pool)
    .await?;
    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, i: &Input) -> AppResult<WatchFolder> {
    get(pool, id).await?;
    let (name, path, tags) = validate(i).await?;
    sqlx::query(
        "UPDATE watch_folders SET organization_id = ?, name = ?, path = ?, tags = ?, delete_after = ?, enabled = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&i.organization_id)
    .bind(name)
    .bind(path)
    .bind(sqlx::types::Json(tags))
    .bind(i.delete_after)
    .bind(i.enabled)
    .bind(now())
    .bind(id)
    .execute(pool)
    .await?;
    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let n = sqlx::query("DELETE FROM watch_folders WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Was eine Datei bei der letzten Runde hatte – nur unveränderte Dateien werden übernommen.
#[derive(Default)]
pub struct Seen(HashMap<PathBuf, (u64, SystemTime)>);

fn ignored(name: &str) -> bool {
    name.starts_with('.')
        || name.starts_with('~')
        || [".part", ".tmp", ".crdownload", ".download"]
            .iter()
            .any(|s| name.ends_with(s))
}

fn supported(name: &str) -> bool {
    let n = name.to_lowercase();
    [".pdf", ".jpg", ".jpeg", ".png", ".webp", ".tif", ".tiff", ".gif", ".bmp"]
        .iter()
        .any(|e| n.ends_with(e))
}

/// Eine Runde für einen Ordner. Gibt die Zahl der übernommenen Dateien zurück.
/// `wait`: nur Dateien übernehmen, die sich seit der letzten Runde nicht geändert haben
/// (aus der Hintergrundschleife); `false` = alles sofort („Jetzt einlesen“).
pub async fn scan(
    pool: &SqlitePool,
    ai: &AiConfig,
    folder: &WatchFolder,
    seen: &mut Seen,
    wait: bool,
) -> Result<usize, String> {
    let root = PathBuf::from(&folder.path);
    let mut dir = tokio::fs::read_dir(&root)
        .await
        .map_err(|e| format!("Ordner {} nicht lesbar: {e}", folder.path))?;
    let mut imported = 0;
    let mut errors = Vec::new();
    while let Ok(Some(entry)) = dir.next_entry().await {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(meta) = entry.metadata().await else { continue };
        if !meta.is_file() || ignored(&name) || !supported(&name) {
            continue;
        }
        let state = (meta.len(), meta.modified().unwrap_or(SystemTime::UNIX_EPOCH));
        if wait {
            let settled = state.1.elapsed().is_ok_and(|age| age >= SETTLE);
            let unchanged = seen.0.get(&path) == Some(&state);
            seen.0.insert(path.clone(), state);
            if !(settled && unchanged) {
                continue;
            }
        }
        seen.0.remove(&path);
        let result = async {
            if meta.len() > MAX_BYTES {
                return Err("Datei ist zu gross (höchstens 200 MB).".to_string());
            }
            let data = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
            let (pdf, pages) = inbox::from_file(&name, data).map_err(|e| e.public_message())?;
            inbox::create(
                pool,
                ai,
                &folder.organization_id,
                folder.created_by.as_deref(),
                &name,
                true,
                "import",
                &format!("Ordner {} · {name}", folder.name),
                &format!("folder:{}", folder.id),
                folder.tags.0.clone(),
                pdf,
                pages,
            )
            .await
            .map_err(|e| e.public_message())?;
            Ok(())
        }
        .await;
        match result {
            Ok(()) => {
                imported += 1;
                if folder.delete_after {
                    let _ = tokio::fs::remove_file(&path).await;
                } else {
                    move_to(&root, DONE_DIR, &path, &name).await;
                }
            }
            Err(e) => {
                errors.push(format!("{name}: {e}"));
                move_to(&root, FAILED_DIR, &path, &name).await;
            }
        }
    }
    sqlx::query(
        "UPDATE watch_folders SET last_check_at = ?, last_error = ?, imported_count = imported_count + ? WHERE id = ?",
    )
    .bind(now())
    .bind(errors.join("; ").chars().take(500).collect::<String>())
    .bind(imported as i64)
    .bind(&folder.id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(imported)
}

/// In einen Unterordner verschieben; gibt es den Namen schon, wird eine Nummer angehängt.
async fn move_to(root: &Path, sub: &str, path: &Path, name: &str) {
    let dir = root.join(sub);
    let _ = tokio::fs::create_dir_all(&dir).await;
    let mut target = dir.join(name);
    let mut n = 1;
    while tokio::fs::metadata(&target).await.is_ok() {
        target = dir.join(format!("{n}-{name}"));
        n += 1;
    }
    if tokio::fs::rename(path, &target).await.is_err() {
        // anderes Dateisystem: kopieren und löschen
        if tokio::fs::copy(path, &target).await.is_ok() {
            let _ = tokio::fs::remove_file(path).await;
        }
    }
}

/// Hintergrund: alle 20 Sekunden jeden aktiven Ordner durchsehen.
pub fn spawn(pool: SqlitePool, config: crate::config::Config, mut shutdown: rocket::Shutdown) {
    tokio::spawn(async move {
        let mut seen: HashMap<String, Seen> = HashMap::new();
        loop {
            let folders: Vec<WatchFolder> = sqlx::query_as("SELECT * FROM watch_folders WHERE enabled = 1")
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
            if !folders.is_empty() {
                let ai = AiConfig::load(&pool, &config).await;
                for f in &folders {
                    let s = seen.entry(f.id.clone()).or_default();
                    if let Err(e) = scan(&pool, &ai, f, s, true).await {
                        tracing::warn!(folder = %f.path, "Ordner-Eingang: {e}");
                        let _ = sqlx::query("UPDATE watch_folders SET last_check_at = ?, last_error = ? WHERE id = ?")
                            .bind(now())
                            .bind(&e)
                            .bind(&f.id)
                            .execute(&pool)
                            .await;
                    }
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(20)) => {}
                _ = &mut shutdown => break,
            }
        }
    });
}

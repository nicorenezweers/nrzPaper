//! Eingangsordner: `data/ingestion/<Organisations-ID>/`. Jede Organisation bekommt dort
//! einen Unterordner. Neue Dateien werden übernommen, sobald sie fertig geschrieben sind,
//! und danach gelöscht – unverschlüsselt bleibt nichts liegen. Was sich nicht übernehmen
//! lässt, landet in `<Organisation>/fehler/`.
//!
//! `notify` meldet Änderungen sofort; zusätzlich wird alle 20 Sekunden nachgeschaut, weil
//! Datei-Ereignisse in Docker-Bind-Mounts nicht immer ankommen.

use crate::config::Config;
use crate::db::models::DocumentSource;
use crate::error::AppError;
use crate::ocr::OcrQueue;
use crate::services::documents::{self, NewDocument};
use notify::{RecursiveMode, Watcher};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const FAILED_DIR: &str = "fehler";
/// So lange darf sich eine Datei nicht mehr ändern, bevor sie übernommen wird.
const SETTLE: Duration = Duration::from_secs(2);
const RESCAN: Duration = Duration::from_secs(20);

/// Was eine Datei bei der letzten Runde hatte – nur unveränderte Dateien werden übernommen.
#[derive(Default)]
pub struct Seen(HashMap<PathBuf, (u64, SystemTime)>);

#[derive(Debug, Default, PartialEq)]
pub struct ScanResult {
    pub imported: usize,
    pub duplicates: usize,
    pub failed: usize,
}

fn ignored(name: &str) -> bool {
    name.starts_with('.')
        || name.starts_with('~')
        || [".part", ".tmp", ".crdownload", ".download"]
            .iter()
            .any(|s| name.ends_with(s))
}

/// Ordner einer neuen Organisation sofort anlegen (nicht erst in der nächsten Runde).
pub async fn create_org_dir(config: &Config, org_id: &str) {
    let dir = config.ingestion_dir().join(org_id);
    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        tracing::warn!("Eingangsordner {} nicht anlegbar: {e}", dir.display());
    }
}

/// Ordner für alle Organisationen anlegen.
pub async fn ensure_dirs(pool: &SqlitePool, root: &Path) -> Result<Vec<String>, AppError> {
    let orgs: Vec<String> =
        sqlx::query_scalar("SELECT id FROM organizations WHERE deleted_at IS NULL")
            .fetch_all(pool)
            .await?;
    for org in &orgs {
        tokio::fs::create_dir_all(root.join(org)).await?;
    }
    Ok(orgs)
}

/// Eine Runde: alle fertigen Dateien übernehmen.
pub async fn scan_once(
    pool: &SqlitePool,
    queue: &OcrQueue,
    root: &Path,
    seen: &mut Seen,
) -> Result<ScanResult, AppError> {
    let mut result = ScanResult::default();
    let orgs = ensure_dirs(pool, root).await?;
    let mut still_there = Vec::new();
    for org in &orgs {
        let mut dir = match tokio::fs::read_dir(root.join(org)).await {
            Ok(d) => d,
            Err(_) => continue,
        };
        while let Ok(Some(entry)) = dir.next_entry().await {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let Ok(meta) = entry.metadata().await else {
                continue;
            };
            if !meta.is_file() || ignored(&name) {
                continue;
            }
            still_there.push(path.clone());
            let state = (
                meta.len(),
                meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            );
            let settled = state.1.elapsed().is_ok_and(|age| age >= SETTLE);
            let unchanged = seen.0.get(&path) == Some(&state);
            seen.0.insert(path.clone(), state);
            if !(settled && unchanged) {
                continue; // wird evtl. noch geschrieben – nächste Runde
            }
            match import(pool, queue, org, &path, &name).await {
                Ok(true) => result.imported += 1,
                Ok(false) => result.duplicates += 1,
                Err(e) => {
                    result.failed += 1;
                    tracing::warn!(file = %path.display(), "Eingangsordner: {e}");
                    let target = root.join(org).join(FAILED_DIR);
                    let _ = tokio::fs::create_dir_all(&target).await;
                    let _ = tokio::fs::rename(&path, target.join(&name)).await;
                }
            }
            seen.0.remove(&path);
        }
    }
    seen.0.retain(|p, _| still_there.contains(p));
    Ok(result)
}

/// Datei übernehmen und löschen. `Ok(false)`: war schon vorhanden (trotzdem gelöscht).
async fn import(
    pool: &SqlitePool,
    queue: &OcrQueue,
    org: &str,
    path: &Path,
    name: &str,
) -> Result<bool, AppError> {
    let data = tokio::fs::read(path).await?;
    let outcome = documents::ingest(
        pool,
        queue,
        NewDocument {
            organization_id: org,
            user_id: None,
            source: DocumentSource::Ingestion,
            file_name: name,
            data,
        },
    )
    .await;
    let imported = match outcome {
        Ok(_) => true,
        Err(AppError::Conflict(_)) => false,
        Err(e) => return Err(e),
    };
    tokio::fs::remove_file(path).await?;
    tracing::info!(
        file = name,
        organization = org,
        duplicate = !imported,
        "Aus dem Eingangsordner übernommen"
    );
    Ok(imported)
}

/// Beobachter starten: sofort bei Datei-Ereignissen, sonst regelmässig.
pub fn spawn(pool: SqlitePool, queue: OcrQueue, config: &Config, mut shutdown: rocket::Shutdown) {
    let root = config.ingestion_dir();
    tokio::spawn(async move {
        if let Err(e) = tokio::fs::create_dir_all(&root).await {
            tracing::error!("Eingangsordner {} nicht anlegbar: {e}", root.display());
            return;
        }
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let watcher = notify::recommended_watcher(move |_event| {
            let _ = tx.send(());
        })
        .and_then(|mut w| w.watch(&root, RecursiveMode::Recursive).map(|_| w));
        if let Err(e) = &watcher {
            tracing::warn!("Eingangsordner wird nur regelmässig geprüft ({e})");
        }
        tracing::info!("Eingangsordner: {}", root.display());
        let mut seen = Seen::default();
        loop {
            match scan_once(&pool, &queue, &root, &mut seen).await {
                Ok(r) if r != ScanResult::default() => tracing::debug!(?r, "Eingangsordner-Runde"),
                Err(e) => tracing::warn!("Eingangsordner: {e}"),
                _ => {}
            }
            // Ein Ereignis weckt sofort; danach kurz warten, bis die Datei fertig ist.
            let wait = if seen.0.is_empty() {
                RESCAN
            } else {
                SETTLE + Duration::from_millis(200)
            };
            tokio::select! {
                _ = rx.recv() => tokio::time::sleep(Duration::from_millis(300)).await,
                _ = tokio::time::sleep(wait) => {}
                _ = &mut shutdown => break,
            }
            while rx.try_recv().is_ok() {}
        }
        drop(watcher);
    });
}

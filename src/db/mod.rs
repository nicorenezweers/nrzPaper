//! Die eine Datenbank: SQLite mit SQLCipher, AES-256 auf Seitenebene. Alles liegt darin –
//! Benutzer, Dokument-Inhalte, OCR-Text, Volltextindex.

pub mod models;

use crate::config::Config;
use crate::error::AppError;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use std::time::Duration;

/// Wie der Schlüssel an SQLCipher geht.
enum Key {
    /// 32 zufällige Bytes als Hex – SQLCipher nimmt ihn direkt, ohne PBKDF2.
    Raw(String),
    /// Passphrase aus der Umgebung – SQLCipher leitet den Schlüssel per PBKDF2 ab.
    Passphrase(String),
}

impl Key {
    /// Wert für `PRAGMA key = …`.
    fn pragma_value(&self) -> String {
        match self {
            Key::Raw(hex) => format!("\"x'{hex}'\""),
            Key::Passphrase(p) => format!("'{}'", p.replace('\'', "''")),
        }
    }
}

fn is_hex_key(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Schlüssel aus `NRZPAPER_DB_KEY`, sonst aus der Schlüsseldatei; beim allerersten Start
/// (noch keine Datenbank) wird die Schlüsseldatei erzeugt.
fn resolve_key(config: &Config) -> Result<Key, AppError> {
    if let Some(k) = config
        .db_key
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty())
    {
        return Ok(if is_hex_key(k) {
            Key::Raw(k.to_lowercase())
        } else {
            Key::Passphrase(k.to_string())
        });
    }
    let path = config.key_path();
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let k = existing.trim();
        if !is_hex_key(k) {
            return Err(AppError::Config(format!(
                "{} enthält keinen gültigen Schlüssel.",
                path.display()
            )));
        }
        return Ok(Key::Raw(k.to_lowercase()));
    }
    if config.database_path().exists() {
        return Err(AppError::Config(
            "Die Datenbank existiert, aber es gibt keinen Schlüssel (NRZPAPER_DB_KEY oder nrzpaper.key).".into(),
        ));
    }
    let key = hex::encode(rand::random::<[u8; 32]>());
    write_private(&path, &key)?;
    tracing::warn!(
        "Neuer Datenbank-Schlüssel in {} – sicher aufbewahren, ohne ihn sind die Daten verloren.",
        path.display()
    );
    Ok(Key::Raw(key))
}

/// Datei nur für den eigenen Benutzer lesbar anlegen.
fn write_private(path: &std::path::Path, contents: &str) -> Result<(), AppError> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(contents.as_bytes())?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, contents)?;
        Ok(())
    }
}

/// Öffnet (bzw. erstellt) die verschlüsselte Datenbank.
pub async fn open(config: &Config) -> Result<SqlitePool, AppError> {
    std::fs::create_dir_all(&config.data_dir)?;
    let key = resolve_key(config)?;
    let options = SqliteConnectOptions::new()
        .filename(config.database_path())
        .create_if_missing(true)
        // Muss die erste Anweisung jeder Verbindung sein.
        .pragma("key", key.pragma_value())
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(10));
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .acquire_timeout(Duration::from_secs(15))
        .connect_with(options)
        .await?;
    // Ein falscher Schlüssel zeigt sich erst beim ersten Lesen.
    sqlx::query("SELECT count(*) FROM sqlite_master")
        .fetch_one(&pool)
        .await
        .map_err(|_| {
            AppError::Config(
                "Datenbank lässt sich nicht entschlüsseln – falscher Schlüssel?".into(),
            )
        })?;
    Ok(pool)
}

/// Schema auf den neuesten Stand bringen. Die Migrationen (`migrations/`) sind ins Programm
/// eingebettet; sqlx merkt sich in `_sqlx_migrations`, was schon gelaufen ist.
pub async fn migrate(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Config(format!("Migration fehlgeschlagen: {e}")))
}

/// Öffnen und migrieren in einem Schritt.
pub async fn connect(config: &Config) -> Result<SqlitePool, AppError> {
    let pool = open(config).await?;
    migrate(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(dir: &std::path::Path, key: Option<&str>) -> Config {
        Config {
            data_dir: dir.to_path_buf(),
            db_key: key.map(String::from),
            ..Config::default()
        }
    }

    #[tokio::test]
    async fn creates_key_file_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let pool = open(&config(dir.path(), None)).await.unwrap();
        sqlx::query("CREATE TABLE t (x TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
        assert!(is_hex_key(
            std::fs::read_to_string(dir.path().join("nrzpaper.key"))
                .unwrap()
                .trim()
        ));
        open(&config(dir.path(), None)).await.unwrap().close().await;
    }

    #[tokio::test]
    async fn wrong_passphrase_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        open(&config(dir.path(), Some("eins")))
            .await
            .unwrap()
            .close()
            .await;
        assert!(open(&config(dir.path(), Some("zwei"))).await.is_err());
    }

    #[test]
    fn missing_key_for_existing_database_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("nrzpaper.db"), b"x").unwrap();
        assert!(resolve_key(&config(dir.path(), None)).is_err());
    }
}

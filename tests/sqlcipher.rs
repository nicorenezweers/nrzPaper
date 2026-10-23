//! Grundannahme von nrzPaper prüfen: eine einzige SQLite-Datei, komplett mit SQLCipher
//! verschlüsselt, mit FTS5-Volltextsuche – angesprochen über sqlx.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::path::Path;

async fn open(path: &Path, passphrase: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        // Muss die erste Anweisung auf jeder Verbindung sein.
        .pragma("key", format!("'{}'", passphrase.replace('\'', "''")))
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new().max_connections(4).connect_with(options).await?;
    // Ein falscher Schlüssel fällt erst beim ersten Lesen auf.
    sqlx::query("SELECT count(*) FROM sqlite_master").fetch_one(&pool).await?;
    Ok(pool)
}

#[tokio::test]
async fn encrypted_database_with_fts5() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nrzpaper.db");

    let pool = open(&path, "richtiges Passwort").await.unwrap();
    let version: String = sqlx::query_scalar("PRAGMA cipher_version").fetch_one(&pool).await.unwrap();
    assert!(!version.is_empty(), "SQLCipher ist nicht aktiv");

    sqlx::query("CREATE VIRTUAL TABLE docs_fts USING fts5(name, content, tokenize = 'unicode61 remove_diacritics 2')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE blobs (id INTEGER PRIMARY KEY, data BLOB NOT NULL)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO docs_fts (name, content) VALUES ('Rechnung Swisscom', 'Geheimer Inhalt: Kundennummer 123456, Zürich')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO blobs (data) VALUES (?)").bind(b"%PDF-1.4 geheimes Dokument".to_vec()).execute(&pool).await.unwrap();

    // Volltextsuche mit Umlaut-Normalisierung
    let hit = sqlx::query("SELECT name FROM docs_fts WHERE docs_fts MATCH 'zurich'").fetch_one(&pool).await.unwrap();
    assert_eq!(hit.get::<String, _>("name"), "Rechnung Swisscom");
    pool.close().await;

    // In der Datei darf nichts lesbar sein – auch nicht der SQLite-Kopf.
    let raw = std::fs::read(&path).unwrap();
    for needle in [&b"SQLite format 3"[..], b"Swisscom", b"123456", b"%PDF", b"geheim"] {
        assert!(raw.windows(needle.len()).all(|w| w != needle), "Klartext in der Datei gefunden: {}", String::from_utf8_lossy(needle));
    }

    // Falscher Schlüssel: kein Zugriff
    assert!(open(&path, "falsch").await.is_err(), "Datei liess sich mit falschem Schlüssel öffnen");

    // Richtiger Schlüssel: alles wieder da
    let pool = open(&path, "richtiges Passwort").await.unwrap();
    let data: Vec<u8> = sqlx::query_scalar("SELECT data FROM blobs").fetch_one(&pool).await.unwrap();
    assert_eq!(data, b"%PDF-1.4 geheimes Dokument");
}

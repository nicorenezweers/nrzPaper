//! Schema-Tests auf der echten, verschlüsselten Datenbank: Migrationen, Volltextindex,
//! Löschketten, Eindeutigkeit, Prüfbedingungen und das Einlesen in die Rust-Modelle.

use nrzpaper::config::Config;
use nrzpaper::db::{self, models::*};
use sqlx::SqlitePool;

async fn setup() -> (tempfile::TempDir, SqlitePool) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        db_key: Some("test".into()),
        ..Config::default()
    };
    let pool = db::connect(&config).await.unwrap();
    (dir, pool)
}

async fn org(pool: &SqlitePool, name: &str) -> String {
    let id = new_id();
    sqlx::query("INSERT INTO organizations (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(now())
        .bind(now())
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn document(
    pool: &SqlitePool,
    org: &str,
    name: &str,
    content: &str,
    sha: &str,
) -> Result<String, sqlx::Error> {
    let id = new_id();
    sqlx::query(
        "INSERT INTO documents (id, organization_id, name, original_name, mime_type, size, sha256, content, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'application/pdf', 10, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(org)
    .bind(name)
    .bind(format!("{name}.pdf"))
    .bind(sha)
    .bind(content)
    .bind(now())
    .bind(now())
    .execute(pool)
    .await?;
    Ok(id)
}

async fn search(pool: &SqlitePool, query: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT d.name FROM documents_fts f JOIN documents d ON d.num = f.rowid
         WHERE documents_fts MATCH ? ORDER BY bm25(documents_fts)",
    )
    .bind(query)
    .fetch_all(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn migrations_create_schema_and_are_idempotent() {
    let (dir, pool) = setup().await;
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .fetch_all(&pool)
            .await
            .unwrap();
    for t in [
        "users",
        "sessions",
        "api_keys",
        "organizations",
        "organization_members",
        "organization_invitations",
        "documents",
        "document_chunks",
        "documents_fts",
        "document_activity",
        "tags",
        "document_tags",
        "tagging_rules",
        "tagging_rule_conditions",
        "tagging_rule_actions",
        "custom_property_definitions",
        "custom_property_options",
        "document_property_values",
        "share_links",
        "webhooks",
        "webhook_events",
        "webhook_deliveries",
        "app_settings",
    ] {
        assert!(tables.iter().any(|x| x == t), "Tabelle {t} fehlt");
    }
    pool.close().await;
    // Zweiter Start: nichts neu anwenden, kein Fehler.
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        db_key: Some("test".into()),
        ..Config::default()
    };
    db::connect(&config).await.unwrap();
    pool.close().await;
}

#[tokio::test]
async fn full_text_index_follows_documents_also_after_vacuum() {
    let (_dir, pool) = setup().await;
    let o = org(&pool, "Privat").await;
    let a = document(
        &pool,
        &o,
        "Rechnung Swisscom",
        "Mobile Abo Oktober, Zürich, CHF 79.90",
        "a",
    )
    .await
    .unwrap();
    document(&pool, &o, "Mietvertrag", "Wohnung in Bern, Kaution", "b")
        .await
        .unwrap();

    assert_eq!(search(&pool, "zurich").await, vec!["Rechnung Swisscom"]);
    assert_eq!(search(&pool, "miet*").await, vec!["Mietvertrag"]);
    assert_eq!(
        search(&pool, "name:rechnung").await,
        vec!["Rechnung Swisscom"]
    );

    // Inhalt geändert → Index folgt
    sqlx::query("UPDATE documents SET content = 'Festnetz Basel' WHERE id = ?")
        .bind(&a)
        .execute(&pool)
        .await
        .unwrap();
    assert!(search(&pool, "zurich").await.is_empty());
    assert_eq!(search(&pool, "basel").await, vec!["Rechnung Swisscom"]);

    // VACUUM darf die Zuordnung nicht zerstören (deshalb die Spalte `num`).
    sqlx::query("DELETE FROM documents WHERE name = 'Mietvertrag'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("VACUUM").execute(&pool).await.unwrap();
    assert_eq!(search(&pool, "basel").await, vec!["Rechnung Swisscom"]);
    assert!(search(&pool, "kaution").await.is_empty());
    sqlx::query("INSERT INTO documents_fts (documents_fts) VALUES ('integrity-check')")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
}

#[tokio::test]
async fn deleting_an_organization_removes_everything_in_it() {
    let (_dir, pool) = setup().await;
    let o = org(&pool, "Gewerbe").await;
    let d = document(&pool, &o, "Offerte", "Text", "x").await.unwrap();
    sqlx::query("INSERT INTO document_chunks (document_id, seq, data) VALUES (?, 0, ?)")
        .bind(&d)
        .bind(vec![1u8; 100])
        .execute(&pool)
        .await
        .unwrap();
    let t = new_id();
    sqlx::query("INSERT INTO tags (id, organization_id, name, created_at, updated_at) VALUES (?, ?, 'Kunde', ?, ?)")
        .bind(&t).bind(&o).bind(now()).bind(now()).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO document_tags (document_id, tag_id, created_at) VALUES (?, ?, ?)")
        .bind(&d)
        .bind(&t)
        .bind(now())
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("DELETE FROM organizations WHERE id = ?")
        .bind(&o)
        .execute(&pool)
        .await
        .unwrap();
    for table in ["documents", "document_chunks", "tags", "document_tags"] {
        // Tabellenname aus der festen Liste oben – kein fremder Text im SQL.
        let n: i64 =
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(n, 0, "{table} nicht leer");
    }
    assert!(search(&pool, "offerte").await.is_empty());
    pool.close().await;
}

#[tokio::test]
async fn uniqueness_rules() {
    let (_dir, pool) = setup().await;
    let a = org(&pool, "A").await;
    let b = org(&pool, "B").await;

    // Gleiche Datei: einmal je Organisation
    document(&pool, &a, "eins", "", "same").await.unwrap();
    assert!(document(&pool, &a, "zwei", "", "same").await.is_err());
    document(&pool, &b, "drei", "", "same").await.unwrap();

    // Tag-Namen ohne Rücksicht auf Gross-/Kleinschreibung eindeutig je Organisation
    let tag = |org: String, name: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query("INSERT INTO tags (id, organization_id, name, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
                .bind(new_id()).bind(org).bind(name).bind(now()).bind(now()).execute(&pool).await
        }
    };
    tag(a.clone(), "Rechnung").await.unwrap();
    assert!(tag(a.clone(), "rechnung").await.is_err());
    tag(b.clone(), "rechnung").await.unwrap();
    pool.close().await;
}

#[tokio::test]
async fn check_constraints_guard_the_data() {
    let (_dir, pool) = setup().await;
    let o = org(&pool, "A").await;
    let d = document(&pool, &o, "doc", "", "s").await.unwrap();
    let u = new_id();
    sqlx::query("INSERT INTO users (id, username, password_hash, created_at, updated_at) VALUES (?, 'nico', 'x', ?, ?)")
        .bind(&u).bind(now()).bind(now()).execute(&pool).await.unwrap();

    // Unbekannte Rolle
    assert!(
        sqlx::query("INSERT INTO organization_members VALUES (?, ?, 'chef', ?)")
            .bind(&o)
            .bind(&u)
            .bind(now())
            .execute(&pool)
            .await
            .is_err()
    );
    // Ungültige Farbe
    assert!(sqlx::query("INSERT INTO tags (id, organization_id, name, color, created_at, updated_at) VALUES (?, ?, 'x', 'rot', 0, 0)")
        .bind(new_id()).bind(&o).execute(&pool).await.is_err());
    // Eigenschaftswert mit zwei gefüllten Spalten
    let def = new_id();
    sqlx::query("INSERT INTO custom_property_definitions (id, organization_id, name, type, created_at, updated_at) VALUES (?, ?, 'Betrag', 'number', 0, 0)")
        .bind(&def).bind(&o).execute(&pool).await.unwrap();
    assert!(sqlx::query("INSERT INTO document_property_values (id, document_id, definition_id, text_value, number_value) VALUES (?, ?, ?, 'x', 1.0)")
        .bind(new_id()).bind(&d).bind(&def).execute(&pool).await.is_err());
    sqlx::query("INSERT INTO document_property_values (id, document_id, definition_id, number_value) VALUES (?, ?, ?, 79.9)")
        .bind(new_id()).bind(&d).bind(&def).execute(&pool).await.unwrap();
    // STRICT: Text in einer Zahlenspalte
    assert!(
        sqlx::query("UPDATE documents SET size = 'gross' WHERE id = ?")
            .bind(&d)
            .execute(&pool)
            .await
            .is_err()
    );
    // Webhook nur mit http(s)
    assert!(sqlx::query("INSERT INTO webhooks (id, organization_id, name, url, created_at, updated_at) VALUES (?, ?, 'x', 'ftp://a', 0, 0)")
        .bind(new_id()).bind(&o).execute(&pool).await.is_err());
    pool.close().await;
}

#[tokio::test]
async fn rows_load_into_models() {
    let (_dir, pool) = setup().await;
    let o = org(&pool, "A").await;
    let d = document(&pool, &o, "Rechnung", "Text", "s").await.unwrap();
    let u = new_id();
    sqlx::query("INSERT INTO users (id, username, password_hash, is_admin, created_at, updated_at) VALUES (?, 'Nico', 'hash', 1, 1, 2)")
        .bind(&u).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO organization_members VALUES (?, ?, 'admin', ?)")
        .bind(&o)
        .bind(&u)
        .bind(now())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO api_keys (id, user_id, name, prefix, key_hash, permissions, created_at) VALUES (?, ?, 'cli', 'np_ab', 'h', '[\"documents:read\",\"tags:create\"]', 0)")
        .bind(new_id()).bind(&u).execute(&pool).await.unwrap();

    let doc: Document = sqlx::query_as("SELECT * FROM documents WHERE id = ?")
        .bind(&d)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc.ocr_status, OcrStatus::Pending);
    assert_eq!(doc.source, DocumentSource::Web);
    assert!(doc.is_pdf() && !doc.in_trash());

    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = 'nico'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(user.is_admin);
    let member: OrganizationMember = sqlx::query_as("SELECT * FROM organization_members")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(member.role, Role::Admin);
    assert!(member.role.can_manage());
    let key: ApiKey = sqlx::query_as("SELECT * FROM api_keys")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        key.permissions.0,
        vec![Permission::DocumentsRead, Permission::TagsCreate]
    );
    pool.close().await;
}

#[tokio::test]
async fn document_chunks_round_trip_and_stay_encrypted() {
    let (dir, pool) = setup().await;
    let o = org(&pool, "A").await;
    let d = document(&pool, &o, "gross", "", "big").await.unwrap();
    // 3,5 MiB in Blöcken zu 1 MiB, mit erkennbarem Muster
    let file: Vec<u8> = b"%PDF-1.7 GEHEIMES-MUSTER "
        .iter()
        .cycle()
        .take(3 * 1024 * 1024 + 512 * 1024)
        .copied()
        .collect();
    for (seq, chunk) in file.chunks(1024 * 1024).enumerate() {
        sqlx::query("INSERT INTO document_chunks (document_id, seq, data) VALUES (?, ?, ?)")
            .bind(&d)
            .bind(seq as i64)
            .bind(chunk)
            .execute(&pool)
            .await
            .unwrap();
    }
    let parts: Vec<Vec<u8>> =
        sqlx::query_scalar("SELECT data FROM document_chunks WHERE document_id = ? ORDER BY seq")
            .bind(&d)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(parts.len(), 4);
    assert_eq!(parts.concat(), file);

    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
    let raw = std::fs::read(dir.path().join("nrzpaper.db")).unwrap();
    let needle = b"GEHEIMES-MUSTER";
    assert!(
        raw.windows(needle.len()).all(|w| w != needle),
        "Dokument-Inhalt lesbar in der Datei"
    );
    pool.close().await;
}

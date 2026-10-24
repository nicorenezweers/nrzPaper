//! Dokumente: Aufnahme, Duplikate, Papierkorb, Texterkennung (echt mit Tesseract/Poppler,
//! falls installiert), Warteschlange mit Volltextsuche, Eingangsordner, Upload und Range.

mod common;

use nrzpaper::auth::api_key;
use nrzpaper::config::Config;
use nrzpaper::db::{self, models::*};
use nrzpaper::error::AppError;
use nrzpaper::ingestion::{self, ScanResult, Seen};
use nrzpaper::ocr::{queue, Engine, OcrQueue};
use nrzpaper::services::documents::{self, Ingested, NewDocument};
use nrzpaper::services::{organizations, users};
use rocket::http::{ContentType, Header, Status};
use sqlx::SqlitePool;
use std::time::{Duration, SystemTime};

const TEXT_PDF: &[u8] = include_bytes!("fixtures/text.pdf");
const SCAN_PDF: &[u8] = include_bytes!("fixtures/scan.pdf");
const SCAN_PNG: &[u8] = include_bytes!("fixtures/scan.png");

async fn setup() -> (tempfile::TempDir, SqlitePool, String) {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        db_key: Some("test".into()),
        ..Config::default()
    };
    let pool = db::connect(&config).await.unwrap();
    let user = users::create(&pool, "nico", "geheim-123", "")
        .await
        .unwrap();
    let org = organizations::create(&pool, &user.id, "Privat")
        .await
        .unwrap();
    (dir, pool, org.id)
}

fn new<'a>(org: &'a str, name: &'a str, data: &[u8]) -> NewDocument<'a> {
    NewDocument {
        organization_id: org,
        user_id: None,
        source: DocumentSource::Web,
        file_name: name,
        data: data.to_vec(),
    }
}

async fn search(pool: &SqlitePool, q: &str) -> Vec<String> {
    sqlx::query_scalar("SELECT d.name FROM documents_fts f JOIN documents d ON d.num = f.rowid WHERE documents_fts MATCH ? AND d.deleted_at IS NULL")
        .bind(q)
        .fetch_all(pool)
        .await
        .unwrap()
}

/// Tesseract und Poppler vorhanden? Sonst werden die OCR-Tests übersprungen.
async fn engine() -> Option<Engine> {
    let e = Engine::detect("deu+eng").await;
    if e.languages.is_none() || !e.poppler {
        eprintln!("Tesseract/Poppler fehlen – OCR-Test übersprungen");
        return None;
    }
    Some(e)
}

#[tokio::test]
async fn ingest_detects_type_and_stores_blocks() {
    let (_d, pool, org) = setup().await;
    let q = OcrQueue::new();
    let doc = documents::ingest(&pool, &q, new(&org, "../geheim/Rechnung.pdf", SCAN_PDF))
        .await
        .unwrap();
    let doc = doc.document();
    assert_eq!(doc.name, "Rechnung.pdf");
    assert_eq!(doc.mime_type, "application/pdf");
    assert_eq!(doc.ocr_status, OcrStatus::Pending);
    assert_eq!(doc.size as usize, SCAN_PDF.len());
    assert_eq!(
        nrzpaper::storage::read_all(&pool, &doc.id).await.unwrap(),
        SCAN_PDF
    );

    // Text-Datei: sofort durchsuchbar
    let txt = documents::ingest(
        &pool,
        &q,
        new(&org, "notiz.txt", "Grüsse aus Zürich".as_bytes()),
    )
    .await
    .unwrap();
    assert_eq!(txt.document().ocr_status, OcrStatus::Done);
    assert_eq!(search(&pool, "zurich").await, vec!["notiz.txt"]);

    // Unbekannter Typ: aufgenommen, aber ohne Erkennung
    let bin = documents::ingest(&pool, &q, new(&org, "daten.bin", &[0xff, 0xfe, 0x00, 0x81]))
        .await
        .unwrap();
    assert_eq!(bin.document().ocr_status, OcrStatus::Skipped);

    // Leere Datei
    assert!(matches!(
        documents::ingest(&pool, &q, new(&org, "leer.pdf", b"")).await,
        Err(AppError::BadRequest(_))
    ));
    pool.close().await;
}

#[tokio::test]
async fn duplicates_trash_and_restore() {
    let (_d, pool, org) = setup().await;
    let q = OcrQueue::new();
    let doc = documents::ingest(&pool, &q, new(&org, "a.pdf", TEXT_PDF))
        .await
        .unwrap()
        .document()
        .clone();
    // Gleiche Datei noch einmal → Konflikt
    assert!(matches!(
        documents::ingest(&pool, &q, new(&org, "b.pdf", TEXT_PDF)).await,
        Err(AppError::Conflict(_))
    ));

    documents::trash(&pool, &org, &doc.id, None).await.unwrap();
    let (active, _) = documents::list(&pool, &org, false, 0, 50).await.unwrap();
    let (trash, n) = documents::list(&pool, &org, true, 0, 50).await.unwrap();
    assert!(active.is_empty() && n == 1 && trash[0].id == doc.id);

    // Erneuter Upload holt sie aus dem Papierkorb
    let again = documents::ingest(&pool, &q, new(&org, "c.pdf", TEXT_PDF))
        .await
        .unwrap();
    assert!(matches!(again, Ingested::Restored(ref d) if d.id == doc.id && !d.in_trash()));

    // Endgültig löschen nur aus dem Papierkorb
    assert!(documents::delete_permanently(&pool, &org, &doc.id)
        .await
        .is_err());
    documents::trash(&pool, &org, &doc.id, None).await.unwrap();
    documents::delete_permanently(&pool, &org, &doc.id)
        .await
        .unwrap();
    let chunks: i64 = sqlx::query_scalar("SELECT count(*) FROM document_chunks")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(chunks, 0);

    // Verlauf
    let events: Vec<String> =
        sqlx::query_scalar("SELECT event FROM document_activity ORDER BY created_at")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(
        events.is_empty(),
        "Verlauf gehört zum gelöschten Dokument: {events:?}"
    );
    pool.close().await;
}

#[tokio::test]
async fn purge_removes_only_old_trash() {
    let (_d, pool, org) = setup().await;
    let q = OcrQueue::new();
    let old = documents::ingest(&pool, &q, new(&org, "alt.pdf", TEXT_PDF))
        .await
        .unwrap()
        .document()
        .id
        .clone();
    let fresh = documents::ingest(&pool, &q, new(&org, "neu.pdf", SCAN_PDF))
        .await
        .unwrap()
        .document()
        .id
        .clone();
    documents::trash(&pool, &org, &old, None).await.unwrap();
    documents::trash(&pool, &org, &fresh, None).await.unwrap();
    sqlx::query("UPDATE documents SET deleted_at = deleted_at - 40 * 86400 WHERE id = ?")
        .bind(&old)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(documents::purge_trash(&pool, 30).await.unwrap(), 1);
    assert!(documents::get(&pool, &org, &fresh).await.is_ok());
    pool.close().await;
}

#[tokio::test]
async fn ocr_reads_text_layer_scans_and_images() {
    let Some(engine) = engine().await else { return };
    let text = engine
        .extract("application/pdf", TEXT_PDF)
        .await
        .unwrap()
        .unwrap();
    assert!(
        text.text.contains("Mietvertrag Wohnung Bern"),
        "{}",
        text.text
    );
    assert_eq!(text.pages, Some(1));

    let scan = engine
        .extract("application/pdf", SCAN_PDF)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(scan.pages, Some(2));
    assert!(scan.text.contains("Rechnung Nr. 4711"), "{}", scan.text);
    assert!(scan.text.contains("Bern"), "Seite 2 fehlt: {}", scan.text);

    let image = engine
        .extract("image/png", SCAN_PNG)
        .await
        .unwrap()
        .unwrap();
    assert!(image.text.contains("Kantonalbank"), "{}", image.text);

    assert!(engine
        .extract("application/zip", b"PK")
        .await
        .unwrap()
        .is_none());
    assert!(engine
        .extract("application/pdf", b"%PDF kaputt")
        .await
        .is_err());
}

#[tokio::test]
async fn queue_processes_and_makes_text_searchable() {
    let Some(engine) = engine().await else { return };
    let (_d, pool, org) = setup().await;
    let q = OcrQueue::new();
    documents::ingest(&pool, &q, new(&org, "Bankbrief.pdf", SCAN_PDF))
        .await
        .unwrap();
    documents::ingest(&pool, &q, new(&org, "kaputt.pdf", b"%PDF-1.4 kaputt"))
        .await
        .unwrap();

    let mut done = 0;
    while let Some(id) = queue::claim_next(&pool).await.unwrap() {
        queue::process(&pool, &engine, &id).await.unwrap();
        done += 1;
    }
    assert_eq!(done, 2);
    assert_eq!(search(&pool, "kantonalbank").await, vec!["Bankbrief.pdf"]);
    assert_eq!(search(&pool, "zurcher").await, vec!["Bankbrief.pdf"]);

    let failed: Document = sqlx::query_as("SELECT * FROM documents WHERE name = 'kaputt.pdf'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(failed.ocr_status, OcrStatus::Failed);
    assert!(failed.ocr_error.is_some());

    // Nach einem Absturz: "processing" wird wieder "pending"
    sqlx::query("UPDATE documents SET ocr_status = 'processing'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(queue::recover(&pool).await.unwrap(), 2);
    pool.close().await;
}

#[tokio::test]
async fn ingestion_folder_imports_settled_files_and_deletes_them() {
    let (dir, pool, org) = setup().await;
    let q = OcrQueue::new();
    let root = dir.path().join("ingestion");
    let mut seen = Seen::default();
    ingestion::scan_once(&pool, &q, &root, &mut seen)
        .await
        .unwrap();
    assert!(root.join(&org).is_dir(), "Ordner der Organisation fehlt");

    let put = |name: &str, data: &[u8]| {
        let path = root.join(&org).join(name);
        std::fs::write(&path, data).unwrap();
        let f = std::fs::File::options().write(true).open(&path).unwrap();
        f.set_modified(SystemTime::now() - Duration::from_secs(10))
            .unwrap();
        path
    };
    let a = put("Brief.pdf", TEXT_PDF);
    put(".versteckt.pdf", SCAN_PDF);
    put("halb.pdf.part", SCAN_PDF);

    // Erste Runde merkt sich die Datei, zweite übernimmt sie (unverändert = fertig geschrieben).
    assert_eq!(
        ingestion::scan_once(&pool, &q, &root, &mut seen)
            .await
            .unwrap(),
        ScanResult::default()
    );
    assert_eq!(
        ingestion::scan_once(&pool, &q, &root, &mut seen)
            .await
            .unwrap(),
        ScanResult {
            imported: 1,
            ..Default::default()
        }
    );
    assert!(!a.exists(), "Datei muss nach der Übernahme gelöscht sein");
    let doc: Document = sqlx::query_as("SELECT * FROM documents WHERE name = 'Brief.pdf'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc.source, DocumentSource::Ingestion);

    // Duplikat wird gelöscht, leere Datei landet in "fehler"
    let dup = put("nochmal.pdf", TEXT_PDF);
    let empty = put("leer.pdf", b"");
    for _ in 0..2 {
        ingestion::scan_once(&pool, &q, &root, &mut seen)
            .await
            .unwrap();
    }
    assert!(!dup.exists() && !empty.exists());
    assert!(root.join(&org).join("fehler").join("leer.pdf").exists());
    // Ignorierte Dateien bleiben unberührt
    assert!(
        root.join(&org).join(".versteckt.pdf").exists()
            && root.join(&org).join("halb.pdf.part").exists()
    );
    pool.close().await;
}

// ------------------------------------------------------------------ über den Server

fn multipart(files: &[(&str, &[u8])]) -> (ContentType, Vec<u8>) {
    let boundary = "nrzpaperGrenze";
    let mut body = Vec::new();
    for (name, data) in files {
        body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: application/octet-stream\r\n\r\n").as_bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (
        ContentType::new("multipart", "form-data").with_params(("boundary", boundary)),
        body,
    )
}

use common::client;

#[rocket::async_test]
async fn upload_and_serve_with_ranges() {
    let (_d, client, org) = client().await;
    {
        let (ct, body) = multipart(&[("Zürich Brief.pdf", TEXT_PDF), ("bild.png", SCAN_PNG)]);
        let res = client
            .post(format!("/api/v1/organizations/{org}/documents"))
            .header(ct.clone())
            .body(body.clone())
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Created);
        let json: serde_json::Value = res.into_json().await.unwrap();
        assert_eq!(json["documents"].as_array().unwrap().len(), 2);
        let id = json["documents"][0]["document"]["id"].as_str().unwrap().to_string();
        assert_eq!(json["documents"][0]["document"]["name"], "Zürich Brief.pdf");
        assert_eq!(json["documents"][0]["document"]["source"], "web", "Sitzung = Oberfläche");
        assert_eq!(json["errors"], serde_json::json!([]));

        // Gleiche Dateien nochmal → Fehler je Datei, zusammen 409
        let res = client
            .post(format!("/api/v1/organizations/{org}/documents"))
            .header(ct.clone())
            .body(body)
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Conflict);
        drop(res);
        // Eine neue und eine bekannte Datei → die neue kommt rein, die andere als Fehler
        let (ct2, body2) = multipart(&[("neu.txt", b"etwas Neues"), ("bild.png", SCAN_PNG)]);
        let res = client
            .post(format!("/api/v1/organizations/{org}/documents"))
            .header(ct2)
            .body(body2)
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Created);
        let json: serde_json::Value = res.into_json().await.unwrap();
        assert_eq!(json["documents"].as_array().unwrap().len(), 1);
        assert_eq!(json["errors"][0]["file"], "bild.png");

        let url = format!("/api/v1/organizations/{org}/documents/{id}/file");
        let res = client.get(url.as_str()).dispatch().await;
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(res.content_type(), Some(ContentType::PDF));
        assert_eq!(res.headers().get_one("Accept-Ranges"), Some("bytes"));
        assert!(res
            .headers()
            .get_one("Content-Disposition")
            .unwrap()
            .starts_with("inline;"));
        assert_eq!(res.into_bytes().await.unwrap(), TEXT_PDF);

        let res = client
            .get(url.as_str())
            .header(Header::new("Range", "bytes=5-14"))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::PartialContent);
        assert_eq!(
            res.headers().get_one("Content-Range"),
            Some(format!("bytes 5-14/{}", TEXT_PDF.len()).as_str())
        );
        assert_eq!(res.into_bytes().await.unwrap(), &TEXT_PDF[5..=14]);

        let res = client
            .get(url.as_str())
            .header(Header::new("Range", "bytes=99999-"))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::RangeNotSatisfiable);

        let res = client.get(format!("{url}?download=true")).dispatch().await;
        assert!(res
            .headers()
            .get_one("Content-Disposition")
            .unwrap()
            .starts_with("attachment;"));

        // Fremde sehen nichts
        users::create(
            client.rocket().state::<SqlitePool>().unwrap(),
            "fremd",
            "geheim-456",
            "",
        )
        .await
        .unwrap();
        common::logout(&client).await;
        assert_eq!(common::login(&client, "fremd", "geheim-456").await, Status::Ok);
        assert_eq!(
            client.get(url.as_str()).dispatch().await.status(),
            Status::NotFound
        );
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn api_document_lifecycle() {
    let (_d, client, org) = client().await;
    {
        let pool = client.rocket().state::<SqlitePool>().unwrap();
        let user = users::by_username(pool, "nico").await.unwrap().unwrap();
        common::logout(&client).await;

        let (_, read_only) =
            api_key::create(pool, &user.id, "lesen", &[Permission::DocumentsRead], None)
                .await
                .unwrap();
        let (_, full) = api_key::create(
            pool,
            &user.id,
            "voll",
            &[
                Permission::DocumentsRead,
                Permission::DocumentsCreate,
                Permission::DocumentsUpdate,
                Permission::DocumentsDelete,
            ],
            None,
        )
        .await
        .unwrap();
        let auth = |k: &str| Header::new("Authorization", format!("Bearer {k}"));
        let base = format!("/api/v1/organizations/{org}/documents");

        let (ct, body) = multipart(&[("Mietvertrag.pdf", TEXT_PDF)]);
        let res = client
            .post(base.as_str())
            .header(auth(&read_only))
            .header(ct.clone())
            .body(body.clone())
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Forbidden);
        let res = client
            .post(base.as_str())
            .header(auth(&full))
            .header(ct)
            .body(body)
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Created);
        let json: serde_json::Value = res.into_json().await.unwrap();
        let id = json["document"]["id"].as_str().unwrap().to_string();
        assert_eq!(
            json["document"]["ocrStatus"]
                .as_str()
                .or(json["document"]["ocr_status"].as_str()),
            Some("pending")
        );

        let list: serde_json::Value = client
            .get(base.as_str())
            .header(auth(&read_only))
            .dispatch()
            .await
            .into_json()
            .await
            .unwrap();
        assert_eq!(list["documentsCount"], 1);

        let res = client
            .patch(format!("{base}/{id}"))
            .header(auth(&full))
            .header(ContentType::JSON)
            .body(r#"{"name":"Vertrag Bern"}"#)
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Ok);
        let one: serde_json::Value = client
            .get(format!("{base}/{id}"))
            .header(auth(&read_only))
            .dispatch()
            .await
            .into_json()
            .await
            .unwrap();
        assert_eq!(one["document"]["name"], "Vertrag Bern");

        let file = client
            .get(format!("{base}/{id}/file"))
            .header(auth(&read_only))
            .dispatch()
            .await;
        assert_eq!(file.into_bytes().await.unwrap(), TEXT_PDF);

        assert_eq!(
            client
                .delete(format!("{base}/{id}"))
                .header(auth(&full))
                .dispatch()
                .await
                .status(),
            Status::NoContent
        );
        let trash: serde_json::Value = client
            .get(format!("{base}/deleted"))
            .header(auth(&read_only))
            .dispatch()
            .await
            .into_json()
            .await
            .unwrap();
        assert_eq!(trash["documentsCount"], 1);
        assert_eq!(
            client
                .post(format!("{base}/{id}/restore"))
                .header(auth(&full))
                .dispatch()
                .await
                .status(),
            Status::NoContent
        );
        assert_eq!(
            client
                .post(format!("{base}/{id}/reprocess"))
                .header(auth(&full))
                .dispatch()
                .await
                .status(),
            Status::Accepted
        );
    }
    client.terminate().await;
}

//! Oberfläche und Suche: Suchsyntax gegen die echte Datenbank, die Aktionen der App über die
//! API (Liste, Suche, Umbenennen, Tags, Papierkorb) und eingebettete Dateien.

mod common;

use nrzpaper::config::Config;
use nrzpaper::db::{self, models::*};
use nrzpaper::ocr::OcrQueue;
use nrzpaper::search;
use nrzpaper::services::documents::{self, NewDocument};
use nrzpaper::services::{organizations, tags, users};
use common::call;
use rocket::http::{ContentType, Status};
use serde_json::json;
use sqlx::SqlitePool;

const TEXT_PDF: &[u8] = include_bytes!("fixtures/text.pdf");
const SCAN_PNG: &[u8] = include_bytes!("fixtures/scan.png");

async fn add(pool: &SqlitePool, org: &str, name: &str, data: &[u8], content: &str) -> String {
    let q = OcrQueue::new();
    let doc = documents::ingest(
        pool,
        &q,
        NewDocument {
            organization_id: org,
            user_id: None,
            source: DocumentSource::Web,
            file_name: name,
            data: data.to_vec(),
        },
    )
    .await
    .unwrap();
    let id = doc.document().id.clone();
    sqlx::query("UPDATE documents SET content = ?, ocr_status = 'done' WHERE id = ?")
        .bind(content)
        .bind(&id)
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn names(pool: &SqlitePool, org: &str, query: &str) -> Vec<String> {
    let (docs, total) = search::search(pool, org, &search::parse(query), 0, 50)
        .await
        .unwrap();
    assert_eq!(
        total as usize,
        docs.len(),
        "Anzahl passt nicht zu den Treffern für {query}"
    );
    let mut n: Vec<String> = docs.into_iter().map(|d| d.name).collect();
    n.sort();
    n
}

#[tokio::test]
async fn search_syntax_against_the_database() {
    let dir = tempfile::tempdir().unwrap();
    let pool = db::connect(&Config {
        data_dir: dir.path().to_path_buf(),
        db_key: Some("t".into()),
        ..Config::default()
    })
    .await
    .unwrap();
    let user = users::create(&pool, "alice", "geheim-123", "")
        .await
        .unwrap();
    let org = organizations::create(&pool, &user.id, "Privat")
        .await
        .unwrap()
        .id;
    let other = organizations::create(&pool, &user.id, "Gewerbe")
        .await
        .unwrap()
        .id;

    let a = add(
        &pool,
        &org,
        "Rechnung Swisscom.pdf",
        b"%PDF-1.4 a",
        "Mobile Abo Oktober, Zürich, CHF 79.90",
    )
    .await;
    let b = add(
        &pool,
        &org,
        "Mietvertrag.pdf",
        b"%PDF-1.4 b",
        "Wohnung in Bern, Kaution drei Monatsmieten",
    )
    .await;
    add(&pool, &org, "Foto.png", SCAN_PNG, "Kantonalbank Beleg").await;
    add(
        &pool,
        &org,
        "notiz.txt",
        "Einkaufsliste Brot".as_bytes(),
        "Einkaufsliste Brot",
    )
    .await;
    add(&pool, &other, "Fremd.pdf", b"%PDF-1.4 c", "Zürich Rechnung").await;

    let steuern = tags::create(&pool, &org, "Steuern", "", "").await.unwrap();
    let wohnen = tags::create(&pool, &org, "Wohnen Bern", "", "")
        .await
        .unwrap();
    tags::add_to_document(&pool, &org, &a, &steuern.id, None)
        .await
        .unwrap();
    tags::add_to_document(&pool, &org, &b, &wohnen.id, None)
        .await
        .unwrap();

    assert_eq!(
        names(&pool, &org, "").await.len(),
        4,
        "andere Organisation darf nicht auftauchen"
    );
    assert_eq!(
        names(&pool, &org, "zurich").await,
        vec!["Rechnung Swisscom.pdf"]
    );
    assert_eq!(
        names(&pool, &org, "kaut").await,
        vec!["Mietvertrag.pdf"],
        "Präfix-Suche"
    );
    assert_eq!(
        names(&pool, &org, "\"drei monatsmieten\"").await,
        vec!["Mietvertrag.pdf"]
    );
    assert!(
        names(&pool, &org, "\"monatsmieten drei\"").await.is_empty(),
        "Wortfolge muss stimmen"
    );
    assert_eq!(
        names(&pool, &org, "name:rechnung").await,
        vec!["Rechnung Swisscom.pdf"]
    );
    assert!(
        names(&pool, &org, "content:swisscom").await.is_empty(),
        "nur im Namen, nicht im Text"
    );
    assert_eq!(
        names(&pool, &org, "tag:steuern").await,
        vec!["Rechnung Swisscom.pdf"]
    );
    assert_eq!(
        names(&pool, &org, "tag:\"wohnen bern\"").await,
        vec!["Mietvertrag.pdf"]
    );
    assert_eq!(
        names(&pool, &org, "has:tags").await,
        vec!["Mietvertrag.pdf", "Rechnung Swisscom.pdf"]
    );
    assert_eq!(
        names(&pool, &org, "-has:tags").await,
        vec!["Foto.png", "notiz.txt"]
    );
    assert_eq!(
        names(&pool, &org, "-tag:steuern type:pdf").await,
        vec!["Mietvertrag.pdf"]
    );
    assert_eq!(names(&pool, &org, "type:image").await, vec!["Foto.png"]);
    assert_eq!(
        names(&pool, &org, "type:text brot").await,
        vec!["notiz.txt"]
    );
    assert_eq!(
        names(&pool, &org, "created:>2000 tag:steuern").await,
        vec!["Rechnung Swisscom.pdf"]
    );
    assert!(names(&pool, &org, "created:<2000").await.is_empty());
    assert_eq!(names(&pool, &org, "status:done").await.len(), 4);

    // Papierkorb taucht nicht auf
    documents::trash(&pool, &org, &b, None).await.unwrap();
    assert!(names(&pool, &org, "kaution").await.is_empty());

    // Böse Eingaben: keine Fehler, keine FTS-Syntax
    for evil in [
        r#"a" OR "b"#,
        "NEAR(zurich",
        "*",
        "^x",
        "col:zurich",
        "\"",
        "'); DROP TABLE documents; --",
        "zur*ch",
    ] {
        search::search(&pool, &org, &search::parse(evil), 0, 10)
            .await
            .unwrap_or_else(|e| panic!("{evil}: {e}"));
    }
    pool.close().await;
}

#[rocket::async_test]
async fn document_actions_as_the_app_does_them() {
    let (_d, client, org) = common::client().await;
    {
        let pool = client.rocket().state::<SqlitePool>().unwrap();
        let pdf = add(
            pool,
            &org,
            "Mietvertrag.pdf",
            TEXT_PDF,
            "Kaution drei Monatsmieten",
        )
        .await;
        add(pool, &org, "Beleg.png", SCAN_PNG, "Kantonalbank").await;
        let base = format!("/api/v1/organizations/{org}");
        let list = |q: &str| {
            let url = format!("{base}/documents?searchQuery={}", q.replace(' ', "%20").replace(':', "%3A"));
            let c = &client;
            async move {
                let (_, v) = call(c, "GET", url, None).await;
                v["documents"].as_array().unwrap().iter().map(|d| d["name"].as_str().unwrap().to_string()).collect::<Vec<_>>()
            }
        };

        // Liste und Suche
        assert_eq!(list("").await.len(), 2);
        assert_eq!(list("kaution").await, ["Mietvertrag.pdf"]);
        assert!(list("tag:x kaution").await.is_empty());

        // Umbenennen, Text, Tag anlegen und anhängen
        let doc = format!("{base}/documents/{pdf}");
        assert_eq!(call(&client, "PATCH", &doc, Some(json!({ "name": "Vertrag Bern" }))).await.0, Status::Ok);
        assert_eq!(
            call(&client, "PATCH", &doc, Some(json!({ "content": "Neuer Text Wohnung" }))).await.0,
            Status::Ok
        );
        let (s, v) = call(&client, "POST", format!("{base}/tags"), Some(json!({ "name": "Wohnen" }))).await;
        assert_eq!(s, Status::Created);
        let tag = v["tag"]["id"].as_str().unwrap().to_string();
        assert_eq!(call(&client, "POST", format!("{doc}/tags"), Some(json!({ "tagId": tag }))).await.0, Status::Created);
        let d = documents::get(pool, &org, &pdf).await.unwrap();
        assert_eq!(
            (d.name.as_str(), d.content.as_str()),
            ("Vertrag Bern", "Neuer Text Wohnung")
        );
        let (_, v) = call(&client, "GET", format!("{doc}/activity"), None).await;
        let events: Vec<&str> = v["activities"].as_array().unwrap().iter().map(|a| a["event"].as_str().unwrap()).collect();
        assert!(events.contains(&"renamed") && events.contains(&"tag_added"), "Verlauf fehlt: {events:?}");
        assert_eq!(list("tag:wohnen").await, ["Vertrag Bern"]);
        assert_eq!(call(&client, "DELETE", format!("{doc}/tags/{tag}"), None).await.0, Status::NoContent);
        assert!(tags::for_documents(pool, std::slice::from_ref(&pdf))
            .await
            .unwrap()
            .is_empty());

        // Papierkorb: hinein, wiederherstellen, wieder hinein, leeren
        assert_eq!(call(&client, "DELETE", &doc, None).await.0, Status::NoContent);
        let (_, v) = call(&client, "GET", format!("{base}/documents/deleted"), None).await;
        assert_eq!(v["documents"][0]["name"], "Vertrag Bern");
        assert_eq!(call(&client, "POST", format!("{doc}/restore"), None).await.0, Status::NoContent);
        assert!(!documents::get(pool, &org, &pdf).await.unwrap().in_trash());
        call(&client, "DELETE", &doc, None).await;
        let (s, v) = call(&client, "POST", format!("{base}/trash/empty"), None).await;
        assert_eq!((s, v["deleted"].clone()), (Status::Ok, json!(1)));
        assert!(
            documents::get(pool, &org, &pdf).await.is_err(),
            "Papierkorb nicht geleert"
        );

        // Tags: anlegen, doppelt → Meldung, ändern, löschen
        let (s, v) = call(
            &client,
            "POST",
            format!("{base}/tags"),
            Some(json!({ "name": "Steuern", "color": "#ff0000", "description": "Belege" })),
        )
        .await;
        assert_eq!(s, Status::Created);
        let steuern = v["tag"]["id"].as_str().unwrap().to_string();
        let (s, v) = call(&client, "POST", format!("{base}/tags"), Some(json!({ "name": "steuern" }))).await;
        assert_eq!(s, Status::Conflict);
        assert!(v.to_string().contains("gibt es schon"));
        assert_eq!(tags::get(pool, &org, &steuern).await.unwrap().color, "#ff0000");
        let (s, _) = call(
            &client,
            "PUT",
            format!("{base}/tags/{steuern}"),
            Some(json!({ "name": "Steuern 2026", "color": "#00aa00" })),
        )
        .await;
        assert_eq!(s, Status::Ok);
        assert_eq!(tags::get(pool, &org, &steuern).await.unwrap().name, "Steuern 2026");
        assert_eq!(call(&client, "DELETE", format!("{base}/tags/{steuern}"), None).await.0, Status::NoContent);
        assert!(tags::get(pool, &org, &steuern).await.is_err());
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn members_cannot_empty_the_trash_of_others_and_assets_are_served() {
    let (_d, client, org) = common::client().await;
    {
        let pool = client.rocket().state::<SqlitePool>().unwrap();
        let member = users::create(pool, "mitglied", "geheim-456", "")
            .await
            .unwrap();
        sqlx::query("INSERT INTO organization_members VALUES (?, ?, 'member', 0)")
            .bind(&org)
            .bind(&member.id)
            .execute(pool)
            .await
            .unwrap();
        common::logout(&client).await;
        common::login(&client, "mitglied", "geheim-456").await;
        assert_eq!(
            call(&client, "POST", format!("/api/v1/organizations/{org}/trash/empty"), None).await.0,
            Status::Forbidden
        );

        // Eingebettete Dateien für Vorschau und Lupe
        for (path, cache) in [
            ("/static/viewer.js", "max-age=3600"),
            ("/static/preview.js", "max-age=3600"),
            ("/static/vendor/pdfjs/pdf.min.mjs", "immutable"),
            ("/static/vendor/pdfjs/pdf.worker.min.mjs", "immutable"),
        ] {
            let res = client.get(path).dispatch().await;
            assert_eq!(res.status(), Status::Ok, "{path}");
            assert_eq!(res.content_type(), Some(ContentType::JavaScript), "{path}");
            assert!(
                res.headers()
                    .get_one("Cache-Control")
                    .unwrap()
                    .contains(cache),
                "{path}"
            );
        }
        assert_eq!(
            client
                .get("/static/viewer.css")
                .dispatch()
                .await
                .content_type(),
            Some(ContentType::CSS)
        );
        assert_eq!(
            client
                .get("/static/../Cargo.toml")
                .dispatch()
                .await
                .status(),
            Status::NotFound
        );

        // Alles eingebettet (App, Schrift, PDF.js) – keine fremden Quellen
        let res = client.get("/login").dispatch().await;
        let csp = res.headers().get_one("Content-Security-Policy").unwrap().to_string();
        assert!(!csp.contains("https://"), "{csp}");
        let page = res.into_string().await.unwrap();
        assert!(!page.contains("https://"), "fremde Quelle in der Seite");
        let font = client
            .get("/static/fonts/inter-latin.woff2")
            .dispatch()
            .await;
        assert_eq!(font.content_type(), Some(ContentType::WOFF2));
        assert!(font
            .headers()
            .get_one("Cache-Control")
            .unwrap()
            .contains("immutable"));
    }
    client.terminate().await;
}

//! Oberfläche und Suche: Suchsyntax gegen die echte Datenbank, Dokumentliste mit
//! Live-Suche, Detailseite, Formular-Aktionen, Tags, Papierkorb und eingebettete Dateien.

use nrzpaper::config::Config;
use nrzpaper::db::{self, models::*};
use nrzpaper::ocr::OcrQueue;
use nrzpaper::search;
use nrzpaper::services::documents::{self, NewDocument};
use nrzpaper::services::{organizations, tags, users};
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
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
    let user = users::create(&pool, "nico", "geheim-123", "")
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

async fn client() -> (tempfile::TempDir, Client, String) {
    let dir = tempfile::tempdir().unwrap();
    let figment = nrzpaper::config::figment()
        .merge(("data_dir", dir.path()))
        .merge(("db_key", "test"))
        .merge(("log_level", "off"))
        .merge(("background", false));
    let client = Client::tracked(nrzpaper::build_with(figment).await.unwrap())
        .await
        .unwrap();
    client
        .post("/register")
        .header(ContentType::Form)
        .body("username=nico&password=geheim-123")
        .dispatch()
        .await;
    let org = {
        let res = client
            .post("/organizations/new")
            .header(ContentType::Form)
            .body("name=Privat")
            .dispatch()
            .await;
        res.headers()
            .get_one("Location")
            .unwrap()
            .trim_start_matches("/organizations/")
            .to_string()
    };
    (dir, client, org)
}

async fn body(client: &Client, uri: String) -> (Status, String) {
    let res = client.get(uri).dispatch().await;
    (res.status(), res.into_string().await.unwrap_or_default())
}

async fn form(client: &Client, uri: String, body: &str) -> Status {
    client
        .post(uri)
        .header(ContentType::Form)
        .body(body.to_string())
        .dispatch()
        .await
        .status()
}

#[rocket::async_test]
async fn pages_and_actions() {
    let (_d, client, org) = client().await;
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
        let img = add(pool, &org, "Beleg.png", SCAN_PNG, "Kantonalbank").await;
        let base = format!("/organizations/{org}");

        // Liste
        let (s, html) = body(&client, base.clone()).await;
        assert_eq!(s, Status::Ok);
        assert!(
            html.contains("Mietvertrag.pdf")
                && html.contains("Beleg.png")
                && html.contains("2 Dokumente")
        );
        assert!(html.contains("data-live-search"), "Live-Suche fehlt");

        // Live-Suche: nur die Ergebnisse, nicht die ganze Seite
        let (_, part) = body(&client, format!("{base}?q=kaution&partial=true")).await;
        assert!(part.contains("Mietvertrag.pdf") && !part.contains("Beleg.png"));
        assert!(
            !part.contains("<html") && !part.contains("<aside"),
            "Teilansicht enthält die ganze Seite"
        );
        assert!(
            part.contains("?find=kaution"),
            "Suchbegriff wird an die Vorschau weitergegeben"
        );
        let (_, filtered) = body(&client, format!("{base}?q=tag%3Ax%20kaution&partial=true")).await;
        assert!(filtered.contains("Keine Treffer"));

        // Detail: PDF-Viewer bzw. Bild-Viewer
        let (s, html) = body(&client, format!("{base}/documents/{pdf}")).await;
        assert_eq!(s, Status::Ok);
        assert!(
            html.contains(r#"data-kind="pdf""#)
                && html.contains("/static/viewer.js?v=")
                && html.contains("Kaution drei Monatsmieten")
        );
        let (_, html) = body(&client, format!("{base}/documents/{img}")).await;
        assert!(html.contains(r#"data-kind="image""#));

        // Aktionen
        let doc = format!("{base}/documents/{pdf}");
        assert_eq!(
            form(&client, format!("{doc}/rename"), "name=Vertrag+Bern").await,
            Status::SeeOther
        );
        assert_eq!(
            form(
                &client,
                format!("{doc}/content"),
                "content=Neuer+Text+Wohnung"
            )
            .await,
            Status::SeeOther
        );
        assert_eq!(
            form(&client, format!("{doc}/tags"), "name=Wohnen").await,
            Status::SeeOther
        );
        let d = documents::get(pool, &org, &pdf).await.unwrap();
        assert_eq!(
            (d.name.as_str(), d.content.as_str()),
            ("Vertrag Bern", "Neuer Text Wohnung")
        );
        let tag = tags::by_name(pool, &org, "wohnen")
            .await
            .unwrap()
            .expect("Tag wurde angelegt");
        let (_, html) = body(&client, doc.clone()).await;
        assert!(
            html.contains("Umbenannt in „Vertrag Bern“")
                && html.contains("Tag „Wohnen“ hinzugefügt"),
            "Verlauf fehlt"
        );
        let (_, list) = body(&client, format!("{base}?q=tag%3Awohnen&partial=true")).await;
        assert!(list.contains("Vertrag Bern"));
        assert_eq!(
            form(&client, format!("{doc}/tags/{}/remove", tag.id), "").await,
            Status::SeeOther
        );
        assert!(tags::for_documents(pool, &[pdf.clone()])
            .await
            .unwrap()
            .is_empty());

        assert_eq!(
            form(&client, format!("{doc}/trash"), "").await,
            Status::SeeOther
        );
        let (_, trash) = body(&client, format!("{base}/trash")).await;
        assert!(trash.contains("Vertrag Bern") && trash.contains("noch 30 Tage"));
        assert_eq!(
            form(&client, format!("{doc}/restore"), "").await,
            Status::SeeOther
        );
        assert!(!documents::get(pool, &org, &pdf).await.unwrap().in_trash());
        assert_eq!(
            form(&client, format!("{doc}/trash"), "").await,
            Status::SeeOther
        );
        assert_eq!(
            form(&client, format!("{base}/trash/empty"), "").await,
            Status::SeeOther
        );
        assert!(
            documents::get(pool, &org, &pdf).await.is_err(),
            "Papierkorb nicht geleert"
        );

        // Tags-Seite: anlegen, doppelt → Meldung, ändern, löschen
        assert_eq!(
            form(
                &client,
                format!("{base}/tags"),
                "name=Steuern&color=%23ff0000&description=Belege"
            )
            .await,
            Status::SeeOther
        );
        let res = client
            .post(format!("{base}/tags"))
            .header(ContentType::Form)
            .body("name=steuern")
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Conflict);
        assert!(res.into_string().await.unwrap().contains("gibt es schon"));
        let steuern = tags::by_name(pool, &org, "Steuern").await.unwrap().unwrap();
        assert_eq!(steuern.color, "#ff0000");
        assert_eq!(
            form(
                &client,
                format!("{base}/tags/{}", steuern.id),
                "name=Steuern+2026&color=nix"
            )
            .await,
            Status::SeeOther
        );
        assert_eq!(
            tags::get(pool, &org, &steuern.id).await.unwrap().name,
            "Steuern 2026"
        );
        assert_eq!(
            form(&client, format!("{base}/tags/{}/delete", steuern.id), "").await,
            Status::SeeOther
        );
        assert!(tags::get(pool, &org, &steuern.id).await.is_err());
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn members_cannot_empty_the_trash_of_others_and_assets_are_served() {
    let (_d, client, org) = client().await;
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
        client.post("/logout").dispatch().await;
        client
            .post("/login")
            .header(ContentType::Form)
            .body("username=mitglied&password=geheim-456")
            .dispatch()
            .await;
        assert_eq!(
            form(&client, format!("/organizations/{org}/trash/empty"), "").await,
            Status::Forbidden
        );

        // Eingebettete Dateien
        for (path, cache) in [
            ("/static/app.js", "max-age=3600"),
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

        // PDF.js ist eingebettet – kein fremdes CDN für Skripte ausser Tailwind
        let csp = client
            .get("/login")
            .dispatch()
            .await
            .headers()
            .get_one("Content-Security-Policy")
            .unwrap()
            .to_string();
        assert!(
            !csp.contains("jsdelivr") && csp.contains("worker-src 'self' blob:"),
            "{csp}"
        );
    }
    client.terminate().await;
}

//! E-Mail-Eingang gegen einen kleinen Schein-IMAP-Server; dazu Ordner, Vorschaubilder und Kacheln.

mod common;

use base64::Engine;
use common::{call, pool};
use rocket::http::{Accept, ContentType, Status};
use rocket::local::asynchronous::Client;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const PAGE: &[u8] = include_bytes!("../assets/test-page.jpg");
const TEXT_PDF: &[u8] = include_bytes!("fixtures/text.pdf");

fn mail(from: &str, subject: &str, parts: &[(&str, &str, &[u8])]) -> Vec<u8> {
    let mut m = format!("From: {from}\r\nTo: nico@example.ch\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"G\"\r\n\r\n--G\r\nContent-Type: text/plain\r\n\r\nHallo\r\n");
    for (name, ctype, data) in parts {
        m.push_str(&format!(
            "--G\r\nContent-Type: {ctype}; name=\"{name}\"\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n",
            base64::engine::general_purpose::STANDARD.encode(data)
        ));
    }
    m.push_str("--G--\r\n");
    m.into_bytes()
}

/// Schein-IMAP (ohne TLS): ein Postfach mit zwei Mails (UID 11 mit PDF und Foto, UID 12 ohne Anhang).
async fn fake_imap(log: Arc<Mutex<Vec<String>>>) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let mails: Vec<(u32, Vec<u8>)> = vec![
        (
            11,
            mail(
                "Swisscom <rechnung@swisscom.ch>",
                "Ihre Rechnung Oktober",
                &[
                    ("Rechnung.pdf", "application/pdf", TEXT_PDF),
                    ("Beleg.jpg", "image/jpeg", PAGE),
                ],
            ),
        ),
        (12, mail("Anna <anna@example.ch>", "Treffen", &[])),
    ];
    tokio::spawn(async move {
        loop {
            let Ok((sock, _)) = listener.accept().await else {
                return;
            };
            let (log, mails) = (log.clone(), mails.clone());
            tokio::spawn(async move {
                let (r, mut w) = sock.into_split();
                let mut lines = BufReader::new(r).lines();
                w.write_all(b"* OK [CAPABILITY IMAP4rev1] Schein-IMAP bereit\r\n")
                    .await
                    .unwrap();
                while let Ok(Some(line)) = lines.next_line().await {
                    let (tag, cmd) = line.split_once(' ').unwrap_or((&line, ""));
                    log.lock().unwrap().push(cmd.to_string());
                    let upper = cmd.to_uppercase();
                    let reply: Vec<u8> = if upper.starts_with("LOGIN") {
                        if cmd.contains("\"geheim\"") {
                            format!("{tag} OK LOGIN completed\r\n").into_bytes()
                        } else {
                            format!("{tag} NO [AUTHENTICATIONFAILED] Invalid credentials\r\n")
                                .into_bytes()
                        }
                    } else if upper.starts_with("SELECT") {
                        format!("* 2 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 7] UIDs valid\r\n* FLAGS (\\Seen)\r\n{tag} OK [READ-WRITE] SELECT completed\r\n").into_bytes()
                    } else if upper.starts_with("UID SEARCH") {
                        format!("* SEARCH 11 12\r\n{tag} OK SEARCH completed\r\n").into_bytes()
                    } else if upper.starts_with("UID FETCH") {
                        let uid: u32 = cmd.split_whitespace().nth(2).unwrap().parse().unwrap();
                        let (n, raw) = mails
                            .iter()
                            .enumerate()
                            .find(|(_, (u, _))| *u == uid)
                            .map(|(i, (_, m))| (i + 1, m.clone()))
                            .unwrap();
                        let mut out =
                            format!("* {n} FETCH (UID {uid} BODY[] {{{}}}\r\n", raw.len())
                                .into_bytes();
                        out.extend_from_slice(&raw);
                        out.extend_from_slice(
                            format!(")\r\n{tag} OK FETCH completed\r\n").as_bytes(),
                        );
                        out
                    } else if upper.starts_with("UID STORE") {
                        let uid: u32 = cmd.split_whitespace().nth(2).unwrap().parse().unwrap();
                        format!(
                            "* 1 FETCH (UID {uid} FLAGS (\\Seen))\r\n{tag} OK STORE completed\r\n"
                        )
                        .into_bytes()
                    } else if upper.starts_with("LOGOUT") {
                        let _ = w
                            .write_all(format!("* BYE\r\n{tag} OK LOGOUT completed\r\n").as_bytes())
                            .await;
                        return;
                    } else {
                        format!("{tag} BAD unbekannt\r\n").into_bytes()
                    };
                    if w.write_all(&reply).await.is_err() {
                        return;
                    }
                }
            });
        }
    });
    port
}

/// Schein-OpenRouter: liest immer dieselbe Rechnung.
async fn fake_ai() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                let mut buf = Vec::new();
                let mut chunk = [0u8; 65536];
                loop {
                    let n = sock.read(&mut chunk).await.unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&buf[..pos]).to_lowercase();
                        let len: usize = head
                            .lines()
                            .find_map(|l| l.strip_prefix("content-length:"))
                            .map(|v| v.trim().parse().unwrap())
                            .unwrap_or(0);
                        if buf.len() >= pos + 4 + len {
                            break;
                        }
                    }
                }
                let content = json!({ "title": "2026-10-15 Swisscom Rechnung", "tags": ["Rechnung"], "metadata": { "absender": "Swisscom" }, "summary": "Rechnung." }).to_string();
                let body = json!({ "model": "mock/vision", "choices": [{ "message": { "content": content } }] }).to_string();
                let _ = sock.write_all(format!("HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}", body.len()).as_bytes()).await;
            });
        }
    });
    url
}

async fn client() -> (tempfile::TempDir, Client, String) {
    let ai = fake_ai().await;
    common::client_with(|f| f.merge(("openrouter_url", ai)).merge(("openrouter_key", "sk-test"))).await
}

async fn post_json(client: &Client, uri: String) -> (Status, Value) {
    call(client, "POST", format!("/api/v1{uri}"), None).await
}

/// Postfach-Eingabe wie aus der Oberfläche.
fn account(org: &str, port: u16, password: &str) -> Value {
    json!({
        "name": "Privat-Mail", "organizationId": org, "host": "127.0.0.1", "port": port,
        "security": "none", "username": "nico@example.ch", "password": password,
        "folder": "INBOX", "tags": ["E-Mail", "Rechnung"], "enabled": true, "markSeen": true,
        "intervalMinutes": 5,
    })
}

async fn new_tag(client: &Client, org: &str, name: &str) -> String {
    let (s, v) = call(client, "POST", format!("/api/v1/organizations/{org}/tags"), Some(json!({ "name": name }))).await;
    assert_eq!(s, Status::Created, "{v}");
    v["tag"]["id"].as_str().unwrap().to_string()
}

async fn items(client: &Client, org: &str) -> Vec<Value> {
    let res = client
        .get(format!("/api/v1/organizations/{org}/review/items"))
        .header(Accept::JSON)
        .dispatch()
        .await;
    let v: Value = res.into_json().await.unwrap();
    v["items"].as_array().unwrap().clone()
}

#[tokio::test]
async fn mail_attachments_land_in_inbox() {
    let (_d, client, org) = client().await;
    let log = Arc::new(Mutex::new(Vec::new()));
    let port = fake_imap(log.clone()).await;
    new_tag(&client, &org, "Rechnung").await;

    // Postfach anlegen (falsches Passwort zuerst, um die Fehlermeldung zu sehen)
    let (st, v) = call(&client, "POST", "/api/v1/me/mail-accounts", Some(account(&org, port, "falsch"))).await;
    assert_eq!(st, Status::Created, "{v}");
    let id = v["mailAccount"]["id"].as_str().unwrap().to_string();
    let pw: String = sqlx::query_scalar("SELECT password FROM mail_accounts")
        .fetch_one(pool(&client))
        .await
        .unwrap();
    assert_eq!(pw, "falsch");
    let (st, v) = post_json(&client, format!("/me/mail-accounts/{id}/test")).await;
    assert_eq!(st, Status::BadRequest);
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Anmeldung abgelehnt"),
        "{v}"
    );

    // Passwort ändern; ohne Passwort bleibt das alte
    let url = format!("/api/v1/me/mail-accounts/{id}");
    assert_eq!(call(&client, "PUT", &url, Some(account(&org, port, "geheim"))).await.0, Status::Ok);
    assert_eq!(call(&client, "PUT", &url, Some(account(&org, port, ""))).await.0, Status::Ok);
    let (st, v) = post_json(&client, format!("/me/mail-accounts/{id}/test")).await;
    assert_eq!(st, Status::Ok, "{v}");

    // Liste zeigt das Postfach, nie das Passwort
    let (_, v) = call(&client, "GET", "/api/v1/me/mail-accounts", None).await;
    let list = v.to_string();
    assert!(list.contains("Privat-Mail") && list.contains("Rechnung") && !list.contains("geheim"), "{list}");

    // Abrufen: zwei Mails, zwei Dokumente (PDF und Foto), keins aus der Mail ohne Anhang
    let (st, v) = post_json(&client, format!("/me/mail-accounts/{id}/fetch")).await;
    assert_eq!(st, Status::Ok, "{v}");
    assert_eq!(v["report"], json!({ "messages": 2, "documents": 2 }));
    let cmds = log.lock().unwrap().clone();
    assert!(
        cmds.iter().any(|c| c.starts_with("UID SEARCH SINCE ")),
        "{cmds:?}"
    );
    assert!(
        cmds.iter().any(|c| c.contains("BODY.PEEK[]")),
        "nur lesen, ohne Seen zu setzen"
    );
    assert_eq!(
        cmds.iter()
            .filter(|c| c.starts_with("UID STORE") && c.contains("\\Seen"))
            .count(),
        2
    );

    let list = items(&client, &org).await;
    assert_eq!(list.len(), 2);
    for i in &list {
        assert_eq!(i["source"], "mail");
        assert_eq!(
            i["sourceInfo"],
            "E-Mail von Swisscom · Ihre Rechnung Oktober"
        );
    }
    let photo = list
        .iter()
        .find(|i| i["pageCount"] == 1)
        .expect("Foto als Seite");
    assert!(
        list.iter().any(|i| i["pageCount"] == 0),
        "PDF ohne Seitenbilder"
    );

    // KI fertig: Name von der KI, Tags vorgegeben + erkannt
    let mut ready = Vec::new();
    for _ in 0..100 {
        ready = items(&client, &org).await;
        if ready.iter().all(|i| i["status"] != "analyse") {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    for i in &ready {
        assert_eq!(i["status"], "pruefen", "{i}");
        assert_eq!(i["name"], "2026-10-15 Swisscom Rechnung");
        assert_eq!(i["tags"], json!(["E-Mail", "Rechnung"]));
    }

    // Noch einmal abrufen: nichts doppelt
    let (_, v) = post_json(&client, format!("/me/mail-accounts/{id}/fetch")).await;
    assert_eq!(v["report"], json!({ "messages": 0, "documents": 0 }));
    let (n, last): (i64, String) =
        sqlx::query_as("SELECT imported_count, last_error FROM mail_accounts")
            .fetch_one(pool(&client))
            .await
            .unwrap();
    assert_eq!((n, last.as_str()), (2, ""));

    // Abnehmen: Dokument mit beiden Tags (E-Mail wird angelegt)
    let res = client
        .post(format!(
            "/api/v1/organizations/{org}/review/{}/approve",
            photo["id"].as_str().unwrap()
        ))
        .header(Accept::JSON)
        .header(ContentType::JSON)
        .body("{}")
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Ok);
    let v: Value = res.into_json().await.unwrap();
    let doc = v["documentId"].as_str().unwrap().to_string();
    let tags: Vec<String> = sqlx::query_scalar("SELECT t.name FROM document_tags dt JOIN tags t ON t.id = dt.tag_id WHERE dt.document_id = ? ORDER BY t.name")
        .bind(&doc)
        .fetch_all(pool(&client))
        .await
        .unwrap();
    assert_eq!(tags, ["E-Mail", "Rechnung"]);

    // Entfernen
    assert_eq!(call(&client, "DELETE", &url, None).await.0, Status::NoContent);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM mail_accounts")
        .fetch_one(pool(&client))
        .await
        .unwrap();
    assert_eq!(n, 0);
    client.terminate().await;
}

#[tokio::test]
async fn mail_account_validation_and_access() {
    let (_d, client, org) = client().await;
    let ok = json!({
        "organizationId": org, "host": "imap.example.ch", "port": 993, "security": "tls",
        "username": "a", "password": "b", "intervalMinutes": 5,
    });
    let with = |k: &str, v: Value| {
        let mut x = ok.clone();
        x[k] = v;
        x
    };
    let create = |body: Value| call(&client, "POST", "/api/v1/me/mail-accounts", Some(body));
    assert_eq!(create(with("host", json!(""))).await.0, Status::BadRequest);
    assert_eq!(create(with("port", json!(70000))).await.0, Status::BadRequest);
    assert_eq!(create(with("security", json!("ssl3"))).await.0, Status::BadRequest);
    assert_eq!(create(with("password", json!(""))).await.0, Status::BadRequest);
    assert_eq!(create(with("organizationId", json!("fremd"))).await.0, Status::NotFound);
    let (st, v) = create(ok.clone()).await;
    assert_eq!(st, Status::Created);
    let id = v["mailAccount"]["id"].as_str().unwrap().to_string();
    // Server nicht erreichbar → verständlicher Fehler, gespeichert am Postfach
    sqlx::query("UPDATE mail_accounts SET host = '127.0.0.1', port = 1")
        .execute(pool(&client))
        .await
        .unwrap();
    let (st, v) = post_json(&client, format!("/me/mail-accounts/{id}/fetch")).await;
    assert_eq!(st, Status::BadRequest);
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("127.0.0.1:1"),
        "{v}"
    );
    let err: String = sqlx::query_scalar("SELECT last_error FROM mail_accounts")
        .fetch_one(pool(&client))
        .await
        .unwrap();
    assert!(!err.is_empty());
    // Fremde Postfächer gibt es nicht
    common::logout(&client).await;
    common::register(&client, "zweite", "geheim-123").await;
    let (st, _) = post_json(&client, format!("/me/mail-accounts/{id}/fetch")).await;
    assert!(st == Status::NotFound || st == Status::Unauthorized, "{st}");
    client.terminate().await;
}

#[tokio::test]
async fn folders_thumbnails_and_grid() {
    let (_d, client, org) = client().await;
    let rechnung = new_tag(&client, &org, "Rechnung").await;
    let steuern = new_tag(&client, &org, "Steuern").await;
    let wohnung = new_tag(&client, &org, "Wohnung").await;

    // Zwei Dokumente hochladen und taggen
    let mut docs = Vec::new();
    for (name, data) in [("rechnung.pdf", TEXT_PDF), ("foto.jpg", PAGE)] {
        let boundary = "G";
        let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: application/octet-stream\r\n\r\n").into_bytes();
        body.extend_from_slice(data);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let res = client
            .post(format!("/api/v1/organizations/{org}/documents"))
            .header(ContentType::new("multipart", "form-data").with_params(("boundary", boundary)))
            .header(Accept::JSON)
            .body(body)
            .dispatch()
            .await;
        let v: Value = res.into_json().await.unwrap();
        docs.push(v["document"]["id"].as_str().unwrap().to_string());
    }
    let (pdf, photo) = (docs[0].clone(), docs[1].clone());
    let base = format!("/api/v1/organizations/{org}");
    for (doc, tag) in [(&pdf, &rechnung), (&photo, &wohnung)] {
        let (s, _) = call(&client, "POST", format!("{base}/documents/{doc}/tags"), Some(json!({ "tagId": tag }))).await;
        assert_eq!(s, Status::Created);
    }

    // Ordner "Finanzen" (Rechnung + Steuern) mit Unterordner "Wohnen" (Wohnung)
    let url = format!("{base}/folders");
    let folder = |body: Value| call(&client, "POST", &url, Some(body));
    let (s, v) = folder(json!({ "name": "Finanzen", "tagIds": [rechnung, steuern] })).await;
    assert_eq!(s, Status::Created, "{v}");
    let fin = v["folder"]["id"].as_str().unwrap().to_string();
    let (s, v) = folder(json!({ "name": "Wohnen", "parentId": fin, "tagIds": [wohnung] })).await;
    assert_eq!(s, Status::Created);
    let wohnen = v["folder"]["id"].as_str().unwrap().to_string();
    assert_eq!(folder(json!({ "name": "" })).await.0, Status::BadRequest);
    assert_eq!(folder(json!({ "name": "X", "tagIds": ["fremd"] })).await.0, Status::NotFound);
    // Ordner kann nicht in sich selbst liegen
    let (s, _) = call(
        &client,
        "PUT",
        format!("{url}/{fin}"),
        Some(json!({ "name": "Finanzen", "parentId": wohnen, "tagIds": [rechnung] })),
    )
    .await;
    assert_eq!(s, Status::BadRequest);

    let names = |folder: String, q: &'static str| {
        let c = &client;
        let base = base.clone();
        async move {
            let (_, v) = call(c, "GET", format!("{base}/documents?folderId={folder}&searchQuery={q}"), None).await;
            let mut n: Vec<String> =
                v["documents"].as_array().unwrap().iter().map(|d| d["name"].as_str().unwrap().to_string()).collect();
            n.sort();
            n
        }
    };
    assert_eq!(names(fin.clone(), "").await, ["foto.jpg", "rechnung.pdf"], "Unterordner zählen mit");
    assert_eq!(names(wohnen.clone(), "").await, ["foto.jpg"]);
    assert_eq!(names(fin.clone(), "foto").await, ["foto.jpg"], "Suche im Ordner");
    let tree = nrzpaper::services::folders::tree(pool(&client), &org)
        .await
        .unwrap();
    assert_eq!(
        tree.iter()
            .map(|f| (f.name.as_str(), f.depth, f.count))
            .collect::<Vec<_>>(),
        [("Finanzen", 0, 2), ("Wohnen", 1, 1)]
    );
    // Seitenleiste: Ordnerbaum in der Übersicht
    let (_, v) = call(&client, "GET", format!("{base}/overview"), None).await;
    assert_eq!(v["folders"].as_array().unwrap().len(), 2);

    // Vorschaubild: Foto immer (image), PDF wenn Poppler da ist, sonst Platzhalter
    let res = client
        .get(format!("{base}/documents/{photo}/thumbnail"))
        .dispatch()
        .await;
    assert_eq!(res.content_type(), Some(ContentType::JPEG));
    let thumb = res.into_bytes().await.unwrap();
    let img = image::load_from_memory(&thumb).unwrap();
    assert!(img.width() <= 420 && img.width() > 100);
    let res = client
        .get(format!("{base}/documents/{pdf}/thumbnail"))
        .dispatch()
        .await;
    let ct = res.content_type().unwrap();
    assert!(ct == ContentType::JPEG || ct == ContentType::SVG, "{ct}");
    drop(res);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM document_thumbnails")
        .fetch_one(pool(&client))
        .await
        .unwrap();
    assert_eq!(n, 2, "einmal erzeugt, gespeichert");

    // Ordner löschen: Unterordner mit, Tags und Dokumente bleiben
    assert_eq!(call(&client, "DELETE", format!("{url}/{fin}"), None).await.0, Status::NoContent);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM folders")
        .fetch_one(pool(&client))
        .await
        .unwrap();
    assert_eq!(n, 0);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM tags")
        .fetch_one(pool(&client))
        .await
        .unwrap();
    assert_eq!(n, 3);
    client.terminate().await;
}

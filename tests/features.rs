//! Erinnerungen, Inbox (alles freigeben, abgelegte leeren, Bilder, Quellen), Ordner-Eingang,
//! Drucken über einen Schein-IPP-Drucker und der Assistent Dieter gegen eine Schein-KI.

mod common;

use common::call;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const PAGE: &[u8] = include_bytes!("../assets/test-page.jpg");
const TEXT_PDF: &[u8] = include_bytes!("fixtures/text.pdf");

/// Kleiner HTTP-Server: zeichnet (Pfad, Body) auf, Antwort aus `answer(pfad, body)`.
async fn serve(
    answer: impl Fn(&str, &[u8]) -> (String, Vec<u8>) + Send + Sync + 'static,
    log: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
) -> String {
    let answer = Arc::new(answer);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let (log, answer) = (log.clone(), answer.clone());
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 65536];
                let (head_end, len) = loop {
                    let n = match sock.read(&mut chunk).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => n,
                    };
                    buf.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&buf[..pos]).to_lowercase();
                        let len = head
                            .lines()
                            .find_map(|l| l.strip_prefix("content-length:"))
                            .map(|v| v.trim().parse::<usize>().unwrap())
                            .unwrap_or(0);
                        break (pos + 4, len);
                    }
                };
                while buf.len() < head_end + len {
                    let n = sock.read(&mut chunk).await.unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
                let path = String::from_utf8_lossy(&buf[..head_end]).split(' ').nth(1).unwrap_or("").to_string();
                let body = buf[head_end..].to_vec();
                let (ctype, reply) = answer(&path, &body);
                log.lock().unwrap().push((path, body));
                let head = format!("HTTP/1.1 200 OK\r\ncontent-type: {ctype}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n", reply.len());
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(&reply).await;
            });
        }
    });
    url
}

/// Schein-KI (OpenAI-kompatibel): Suchbegriffe, Antworten und Dokument-Analyse.
async fn fake_ai(log: Arc<Mutex<Vec<(String, Vec<u8>)>>>) -> String {
    serve(
        |_, body| {
            let req: Value = serde_json::from_slice(body).unwrap_or_default();
            let system = req["messages"][0]["content"].as_str().unwrap_or("");
            let content = if system.contains("Suchanfragen") {
                r#"{"queries": ["Versicherungsnummer"]}"#.to_string()
            } else if system.contains("Dieter") {
                r#"{"answer": "Deine Versicherungsnummer ist **756.1234.5678.97** [D1].", "sources": ["D1"]}"#.to_string()
            } else {
                r#"{"title": "2026-10-15 Beleg", "tags": [], "metadata": {}, "summary": "Beleg"}"#.to_string()
            };
            let reply = json!({ "model": "mock/text", "choices": [{ "message": { "content": content } }] });
            ("application/json".into(), reply.to_string().into_bytes())
        },
        log,
    )
    .await
}

async fn upload(c: &Client, uri: String, name: &str, data: &[u8]) -> (Status, Value) {
    let b = "G";
    let mut body = format!("--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: application/octet-stream\r\n\r\n").into_bytes();
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{b}--\r\n").as_bytes());
    let res = c
        .post(uri)
        .header(ContentType::new("multipart", "form-data").with_params(("boundary", b)))
        .body(body)
        .dispatch()
        .await;
    let s = res.status();
    (s, res.into_json().await.unwrap_or(Value::Null))
}

async fn text_doc(c: &Client, org: &str, name: &str, text: &str) -> String {
    let (s, v) = upload(c, format!("/api/v1/organizations/{org}/documents"), name, text.as_bytes()).await;
    assert_eq!(s, Status::Created, "{v}");
    v["document"]["id"].as_str().unwrap().to_string()
}

#[rocket::async_test]
async fn reminders_lifecycle() {
    let (_d, c, org) = common::client().await;
    let doc = text_doc(&c, &org, "vertrag.txt", "Mietvertrag kündigen bis Ende Jahr").await;
    let base = format!("/api/v1/organizations/{org}");

    let (s, v) = call(&c, "POST", format!("{base}/documents/{doc}/reminders"), Some(json!({ "days": 10, "note": "kündigen" }))).await;
    assert_eq!(s, Status::Created, "{v}");
    let id = v["reminder"]["id"].as_str().unwrap().to_string();
    assert_eq!(v["reminder"]["documentName"], "vertrag.txt");
    assert!(call(&c, "POST", format!("{base}/documents/{doc}/reminders"), Some(json!({ "dueAt": 1 }))).await.0 == Status::BadRequest);
    assert!(call(&c, "POST", format!("{base}/documents/{doc}/reminders"), Some(json!({}))).await.0 == Status::BadRequest);

    // Noch nicht fällig
    let (_, o) = call(&c, "GET", format!("{base}/overview"), None).await;
    assert_eq!(o["remindersDue"], 0);
    // Fällig machen (direkt in der Datenbank), dann zählt sie
    sqlx::query("UPDATE reminders SET due_at = 1").execute(common::pool(&c)).await.unwrap();
    let (_, o) = call(&c, "GET", format!("{base}/overview"), None).await;
    assert_eq!(o["remindersDue"], 1);
    let (_, l) = call(&c, "GET", format!("{base}/reminders"), None).await;
    assert_eq!(l["reminders"][0]["note"], "kündigen");

    // Verschieben öffnet und setzt neu, erledigt nimmt sie aus der Liste
    let (_, v) = call(&c, "PUT", format!("/api/v1/reminders/{id}"), Some(json!({ "snoozeDays": 1 }))).await;
    assert!(v["reminder"]["dueAt"].as_i64().unwrap() > 1);
    call(&c, "PUT", format!("/api/v1/reminders/{id}"), Some(json!({ "done": true }))).await;
    let (_, l) = call(&c, "GET", format!("{base}/reminders"), None).await;
    assert_eq!(l["reminders"], json!([]));
    let (_, l) = call(&c, "GET", format!("{base}/documents/{doc}/reminders"), None).await;
    assert!(l["reminders"][0]["doneAt"].is_i64());

    // Fremde sehen sie nicht
    call(&c, "POST", "/api/v1/settings/users", Some(json!({ "username": "fremd", "password": "geheim-456" }))).await;
    common::logout(&c).await;
    common::login(&c, "fremd", "geheim-456").await;
    assert_eq!(call(&c, "PUT", format!("/api/v1/reminders/{id}"), Some(json!({ "done": false }))).await.0, Status::NotFound);
    assert_eq!(call(&c, "DELETE", format!("/api/v1/reminders/{id}"), None).await.0, Status::NotFound);
    c.terminate().await;
}

#[rocket::async_test]
async fn inbox_approve_all_clear_and_folders() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let ai = fake_ai(log).await;
    let (dir, c, org) = common::client_with(|f| f.merge(("openrouter_url", ai)).merge(("openrouter_key", "sk-test"))).await;
    let base = format!("/api/v1/organizations/{org}");

    // Bild und PDF in die Inbox; anderes wird abgelehnt
    let (s, v) = upload(&c, format!("{base}/review/import"), "beleg.jpg", PAGE).await;
    assert_eq!(s, Status::Ok, "{v}");
    assert_eq!(v["items"][0]["pageCount"], 1, "Bild als Seite");
    let (s, _) = upload(&c, format!("{base}/review/import"), "brief.pdf", TEXT_PDF).await;
    assert_eq!(s, Status::Ok);
    let (s, _) = upload(&c, format!("{base}/review/import"), "notiz.txt", b"nur Text").await;
    assert_eq!(s, Status::BadRequest);

    let wait = || async {
        for _ in 0..100 {
            let (_, v) = call(&c, "GET", format!("{base}/review/items"), None).await;
            if v["items"].as_array().unwrap().iter().all(|i| i["status"] != "analyse") {
                return v;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("KI wurde nicht fertig");
    };
    let v = wait().await;
    assert_eq!(v["items"].as_array().unwrap().len(), 2);

    // Ordner-Eingang: Ordner anlegen, Datei hineinlegen, einlesen → Inbox mit Quelle
    let inbox_dir = dir.path().join("scans");
    std::fs::create_dir_all(&inbox_dir).unwrap();
    std::fs::write(inbox_dir.join("vom-drucker.pdf"), b"%PDF-1.4 kaputt aber pdf").unwrap();
    std::fs::write(inbox_dir.join("liesmich.txt"), b"bleibt liegen").unwrap();
    assert_eq!(
        call(&c, "POST", "/api/v1/settings/watch-folders", Some(json!({ "organizationId": org, "name": "Drucker", "path": "relativ" }))).await.0,
        Status::BadRequest
    );
    let (s, v) = call(
        &c,
        "POST",
        "/api/v1/settings/watch-folders",
        Some(json!({ "organizationId": org, "name": "Drucker", "path": inbox_dir.to_str().unwrap(), "tags": ["Scan"] })),
    )
    .await;
    assert_eq!(s, Status::Created, "{v}");
    let folder = v["watchFolder"]["id"].as_str().unwrap().to_string();
    let (_, v) = call(&c, "POST", format!("/api/v1/settings/watch-folders/{folder}/scan"), None).await;
    assert_eq!(v["imported"], 1, "{v}");
    assert!(inbox_dir.join("nrzpaper-importiert/vom-drucker.pdf").exists(), "verschoben");
    assert!(inbox_dir.join("liesmich.txt").exists(), "andere Dateien bleiben");
    let v = wait().await;
    let from_folder = v["items"].as_array().unwrap().iter().find(|i| i["sourceRef"] == format!("folder:{folder}")).cloned().unwrap();
    assert_eq!(from_folder["tags"], json!(["Scan"]));
    assert!(from_folder["sourceInfo"].as_str().unwrap().contains("Drucker"));
    let (_, s) = call(&c, "GET", format!("{base}/inbox-sources"), None).await;
    assert_eq!(s["folders"][0]["ref"], format!("folder:{folder}"));

    // Alle freigeben: drei Einträge werden zu Dokumenten
    let (s, v) = call(&c, "POST", format!("{base}/review/approve-all"), None).await;
    assert_eq!(s, Status::Ok);
    assert_eq!(v["approved"], 3, "{v}");
    let (_, docs) = call(&c, "GET", format!("{base}/documents"), None).await;
    assert_eq!(docs["documentsCount"], 3);

    // „Zuletzt abgelegt“ leeren – Dokumente bleiben
    let (_, v) = call(&c, "DELETE", format!("{base}/review-filed"), None).await;
    assert_eq!(v["removed"], 3);
    let (_, v) = call(&c, "GET", format!("{base}/review/items"), None).await;
    assert_eq!(v["items"], json!([]));
    let (_, docs) = call(&c, "GET", format!("{base}/documents"), None).await;
    assert_eq!(docs["documentsCount"], 3);
    c.terminate().await;
}

/// Schein-Drucker: kann nur JPEG (wie viele Tintenstrahler), meldet sich als „Testdrucker“.
fn ipp_reply(path: &str, body: &[u8]) -> (String, Vec<u8>) {
    let op = u16::from_be_bytes([body[2], body[3]]);
    let mut r = vec![2, 0, 0, 0, body[4], body[5], body[6], body[7], 0x01];
    let mut attr = |tag: u8, name: &str, val: &[u8]| {
        r.push(tag);
        r.extend_from_slice(&(name.len() as u16).to_be_bytes());
        r.extend_from_slice(name.as_bytes());
        r.extend_from_slice(&(val.len() as u16).to_be_bytes());
        r.extend_from_slice(val);
    };
    attr(0x47, "attributes-charset", b"utf-8");
    attr(0x48, "attributes-natural-language", b"en");
    r.push(if op == 0x000B { 0x04 } else { 0x02 });
    let mut attr = |tag: u8, name: &str, val: &[u8]| {
        r.push(tag);
        r.extend_from_slice(&(name.len() as u16).to_be_bytes());
        r.extend_from_slice(name.as_bytes());
        r.extend_from_slice(&(val.len() as u16).to_be_bytes());
        r.extend_from_slice(val);
    };
    if op == 0x000B {
        attr(0x42, "printer-name", b"Testdrucker");
        attr(0x41, "printer-make-and-model", b"Schein IPP 1");
        attr(0x49, "document-format-supported", b"image/jpeg");
        attr(0x49, "", b"image/urf");
        attr(0x23, "printer-state", &3i32.to_be_bytes());
        attr(0x22, "color-supported", &[1]);
        attr(0x44, "sides-supported", b"one-sided");
        attr(0x44, "", b"two-sided-long-edge");
    } else {
        attr(0x21, "job-id", &42i32.to_be_bytes());
    }
    r.push(0x03);
    let _ = path;
    ("application/ipp".into(), r)
}

#[rocket::async_test]
async fn print_on_network_printer() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let printer = serve(ipp_reply, log.clone()).await;
    let (_d, c, org) = common::client().await;
    let host = printer.trim_start_matches("http://");

    let (s, v) = call(&c, "POST", "/api/v1/settings/printers", Some(json!({ "url": format!("ipp://{host}/ipp/print") }))).await;
    assert_eq!(s, Status::Created, "{v}");
    assert_eq!(v["printer"]["name"], "Testdrucker");
    let pid = v["printer"]["id"].as_str().unwrap().to_string();
    assert_eq!(
        call(&c, "POST", "/api/v1/settings/printers", Some(json!({ "url": format!("ipp://{host}/ipp/print") }))).await.0,
        Status::Conflict
    );
    let (_, st) = call(&c, "GET", format!("/api/v1/printers/{pid}/status"), None).await;
    assert_eq!((st["ok"].clone(), st["info"]["state"].clone(), st["info"]["duplex"].clone()), (json!(true), json!("idle"), json!(true)));

    // Ein Bild drucken: geht als JPEG, mit Kopien und beidseitig
    let (_, v) = upload(&c, format!("/api/v1/organizations/{org}/documents"), "foto.jpg", PAGE).await;
    let doc = v["document"]["id"].as_str().unwrap().to_string();
    let (s, v) = call(
        &c,
        "POST",
        format!("/api/v1/organizations/{org}/documents/{doc}/print"),
        Some(json!({ "printerId": pid, "copies": 2, "duplex": true })),
    )
    .await;
    assert_eq!(s, Status::Ok, "{v}");
    assert_eq!(v["jobs"], json!([42]));
    let (_, job) = log.lock().unwrap().iter().rev().find(|(_, b)| b.len() > 4 && b[3] == 0x02).cloned().unwrap();
    let text = String::from_utf8_lossy(&job);
    assert!(text.contains("image/jpeg") && text.contains("two-sided-long-edge") && text.contains("copies"));
    assert!(job.windows(3).any(|w| w == [0xFF, 0xD8, 0xFF]), "JPEG im Auftrag");

    // Text kann dieser Drucker nicht
    let txt = text_doc(&c, &org, "notiz.txt", "Hallo").await;
    let (s, v) = call(&c, "POST", format!("/api/v1/organizations/{org}/documents/{txt}/print"), Some(json!({ "printerId": pid }))).await;
    assert_eq!(s, Status::BadRequest, "{v}");
    assert_eq!(
        call(&c, "POST", format!("/api/v1/organizations/{org}/documents/{doc}/print"), Some(json!({ "printerId": pid, "copies": 0 }))).await.0,
        Status::BadRequest
    );
    let (_, a) = call(&c, "GET", format!("/api/v1/organizations/{org}/documents/{doc}/activity"), None).await;
    assert!(a.to_string().contains("printed"));
    c.terminate().await;
}

#[rocket::async_test]
async fn dieter_answers_with_sources() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let ai = fake_ai(log.clone()).await;
    let (_d, c, org) = common::client_with(|f| f.merge(("openrouter_url", ai)).merge(("openrouter_key", "sk-test"))).await;
    let id = text_doc(&c, &org, "police.txt", &format!("{}Versicherungsnummer: 756.1234.5678.97", "Allgemeine Bedingungen. ".repeat(50))).await;
    text_doc(&c, &org, "anderes.txt", "Nichts Wichtiges").await;
    let (s, v) = call(
        &c,
        "POST",
        format!("/api/v1/organizations/{org}/assistant"),
        Some(json!({ "messages": [{ "role": "user", "content": "Wie lautet meine Versicherungsnummer?" }] })),
    )
    .await;
    assert_eq!(s, Status::Ok, "{v}");
    assert_eq!(v["answer"], "Deine Versicherungsnummer ist **756.1234.5678.97**.");
    assert_eq!(v["sources"][0]["id"], id.as_str());
    assert_eq!(v["queries"], json!(["Versicherungsnummer"]));
    // Die KI hat die Textstelle bekommen
    let sent = log.lock().unwrap().iter().map(|(_, b)| String::from_utf8_lossy(b).into_owned()).collect::<Vec<_>>().join("\n");
    assert!(sent.contains("756.1234.5678.97"));
    assert_eq!(
        call(&c, "POST", format!("/api/v1/organizations/{org}/assistant"), Some(json!({ "messages": [] }))).await.0,
        Status::BadRequest
    );
    c.terminate().await;
}

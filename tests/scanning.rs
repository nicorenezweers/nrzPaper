//! Scannen und Prüfen (aus nrzDocsScanner): echter eSCL-Ablauf gegen einen kleinen Schein-Scanner,
//! KI-Analyse gegen ein Schein-OpenRouter, Prüfen, Freigeben → Dokument mit Tags und Eigenschaften.

use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const PAGE: &[u8] = include_bytes!("../assets/test-page.jpg");
const TEXT_PDF: &[u8] = include_bytes!("fixtures/text.pdf");

type Handler =
    Arc<dyn Fn(&str, &str, &[u8]) -> (u16, Vec<(String, String)>, Vec<u8>) + Send + Sync>;

/// Mini-HTTP-Server für Tests: (Methode, Pfad, Body) → (Status, Kopfzeilen, Body).
async fn serve(handler: Handler) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let handler = handler.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 8192];
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
                let line = String::from_utf8_lossy(
                    &buf[..buf.iter().position(|&b| b == b'\r').unwrap_or(0)],
                )
                .to_string();
                let mut parts = line.split(' ');
                let (method, path) = (
                    parts.next().unwrap_or("").to_string(),
                    parts.next().unwrap_or("").to_string(),
                );
                let (status, headers, body) = handler(
                    &method,
                    &path,
                    &buf[head_end..(head_end + len).min(buf.len())],
                );
                let mut out = format!(
                    "HTTP/1.1 {status} X\r\ncontent-length: {}\r\nconnection: close\r\n",
                    body.len()
                );
                for (k, v) in headers {
                    out.push_str(&format!("{k}: {v}\r\n"));
                }
                out.push_str("\r\n");
                let _ = sock.write_all(out.as_bytes()).await;
                let _ = sock.write_all(&body).await;
            });
        }
    });
    url
}

/// Schein-Scanner mit eSCL: Status, Auftrag anlegen, Seite abholen (beim ersten Mal 503 = noch nicht fertig).
async fn fake_scanner(jobs: Arc<Mutex<Vec<String>>>) -> String {
    serve(Arc::new(move |method, path, body| match (method, path) {
        ("GET", "/eSCL/ScannerStatus") => (
            200,
            vec![],
            b"<scan:ScannerStatus><pwg:State>Idle</pwg:State></scan:ScannerStatus>".to_vec(),
        ),
        ("POST", "/eSCL/ScanJobs") => {
            let settings = String::from_utf8_lossy(body).to_string();
            let mut jobs = jobs.lock().unwrap();
            jobs.push(settings);
            let id = format!("job-{}", jobs.len());
            (
                201,
                vec![("Location".into(), format!("/eSCL/ScanJobs/{id}"))],
                vec![],
            )
        }
        ("GET", p) if p.ends_with("/NextDocument") => {
            let mut jobs = jobs.lock().unwrap();
            // erste Abfrage je Auftrag: noch beschäftigt
            if !jobs.iter().any(|j| j == p) {
                jobs.push(p.to_string());
                (503, vec![], vec![])
            } else {
                (
                    200,
                    vec![("Content-Type".into(), "image/jpeg".into())],
                    PAGE.to_vec(),
                )
            }
        }
        _ => (404, vec![], vec![]),
    }))
    .await
}

/// Schein-OpenRouter: merkt sich die Anfragen, antwortet mit `answer` (oder Fehler, wenn None).
async fn fake_ai(requests: Arc<Mutex<Vec<Value>>>, answer: Arc<Mutex<Option<Value>>>) -> String {
    serve(Arc::new(move |method, path, body| {
        if method != "POST" || path != "/chat/completions" {
            return (404, vec![], vec![]);
        }
        let req: Value = serde_json::from_slice(body).unwrap_or_default();
        requests.lock().unwrap().push(req.clone());
        match answer.lock().unwrap().clone() {
            Some(a) => {
                let content = format!("```json\n{a}\n```");
                let body = json!({ "model": "mock/vision", "choices": [{ "message": { "content": content } }] });
                (200, vec![("Content-Type".into(), "application/json".into())], body.to_string().into_bytes())
            }
            None => (429, vec![], json!({ "error": { "message": "rate limited" } }).to_string().into_bytes()),
        }
    }))
    .await
}

struct Env {
    _dir: tempfile::TempDir,
    client: Client,
    org: String,
    ai_requests: Arc<Mutex<Vec<Value>>>,
    ai_answer: Arc<Mutex<Option<Value>>>,
    scan_jobs: Arc<Mutex<Vec<String>>>,
}

fn swisscom() -> Value {
    json!({
        "title": "2026-10-15 Swisscom Rechnung Mobile",
        "tags": ["rechnung", "Erfunden"],
        "metadata": { "absender": "Swisscom", "betrag": "79.90", "datum": "2026-10-15", "waehrung": "CHF", "iban": "" },
        "summary": "Mobile-Rechnung für Oktober."
    })
}

async fn env() -> Env {
    let ai_requests = Arc::new(Mutex::new(Vec::new()));
    let ai_answer = Arc::new(Mutex::new(Some(swisscom())));
    let scan_jobs = Arc::new(Mutex::new(Vec::new()));
    let ai_url = fake_ai(ai_requests.clone(), ai_answer.clone()).await;
    let scanner = fake_scanner(scan_jobs.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let figment = nrzpaper::config::figment()
        .merge(("data_dir", dir.path()))
        .merge(("db_key", "test"))
        .merge(("log_level", "off"))
        .merge(("background", false))
        .merge(("scanner_url", scanner))
        .merge(("openrouter_url", ai_url))
        .merge(("openrouter_key", "sk-test"));
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
    Env {
        _dir: dir,
        client,
        org,
        ai_requests,
        ai_answer,
        scan_jobs,
    }
}

impl Env {
    fn pool(&self) -> &SqlitePool {
        self.client.rocket().state::<SqlitePool>().unwrap()
    }

    async fn json(&self, method: &str, uri: String, body: Option<Value>) -> (Status, Value) {
        let req = match method {
            "GET" => self.client.get(uri),
            "POST" => self.client.post(uri),
            "PUT" => self.client.put(uri),
            "DELETE" => self.client.delete(uri),
            _ => unreachable!(),
        }
        .header(rocket::http::Accept::JSON);
        let req = match body {
            Some(b) => req.header(ContentType::JSON).body(b.to_string()),
            None => req,
        };
        let res = req.dispatch().await;
        let status = res.status();
        (
            status,
            res.into_json::<Value>().await.unwrap_or(Value::Null),
        )
    }

    async fn form(&self, uri: String, body: &str) -> Status {
        self.client
            .post(uri)
            .header(ContentType::Form)
            .body(body)
            .dispatch()
            .await
            .status()
    }

    /// Warten, bis die KI fertig ist (Status nicht mehr "analyse").
    async fn wait_reviewed(&self, org: &str, id: &str) -> Value {
        for _ in 0..100 {
            let (_, v) = self
                .json("GET", format!("/organizations/{org}/review/items"), None)
                .await;
            if let Some(item) = v["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| i["id"] == id)
            {
                if item["status"] != "analyse" {
                    return item.clone();
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("KI wurde nicht fertig");
    }
}

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

#[tokio::test]
async fn scan_review_and_approve() {
    let e = env().await;
    let org = e.org.clone();
    let scan = format!("/organizations/{org}/scan");
    assert_eq!(
        e.form(
            format!("/organizations/{org}/tags"),
            "name=Rechnung&description=Rechnungen+aller+Art"
        )
        .await,
        Status::SeeOther
    );
    assert_eq!(
        e.form(format!("/organizations/{org}/tags"), "name=Privat")
            .await,
        Status::SeeOther
    );

    // Seiten sind da und bedienbar
    let html = e
        .client
        .get(scan.clone())
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert!(html.contains("data-scan-page") && html.contains("/static/scan.js"));
    let (_, s) = e.json("GET", "/scanner/status".into(), None).await;
    assert_eq!(s["ok"], true);
    assert_eq!(s["state"], "Idle");

    // Zwei Seiten scannen – Graustufen, 150 dpi kommen beim Scanner an
    let (st, v) = e
        .json(
            "POST",
            format!("{scan}/page"),
            Some(json!({ "color": false, "dpi": 150 })),
        )
        .await;
    assert_eq!(st, Status::Ok, "{v}");
    let (_, v) = e
        .json("POST", format!("{scan}/page"), Some(json!({})))
        .await;
    let pages: Vec<String> = serde_json::from_value(v["session"]["pages"].clone()).unwrap();
    assert_eq!(pages.len(), 2);
    let first_job = e.scan_jobs.lock().unwrap()[0].clone();
    assert!(
        first_job.contains("Grayscale8") && first_job.contains("<scan:XResolution>150<"),
        "{first_job}"
    );
    assert_eq!(v["tags"].as_array().unwrap().len(), 2);

    // Seitenbild, Umsortieren, falsche Reihenfolge, Foto anhängen, Seite löschen
    let img = e
        .client
        .get(format!("{scan}/pages/{}", pages[0]))
        .dispatch()
        .await;
    assert_eq!(img.content_type(), Some(ContentType::JPEG));
    assert_eq!(img.into_bytes().await.unwrap(), PAGE);
    let reversed = vec![pages[1].clone(), pages[0].clone()];
    let (_, v) = e
        .json(
            "PUT",
            format!("{scan}/session"),
            Some(json!({ "pages": reversed, "hintTags": ["Privat"] })),
        )
        .await;
    assert_eq!(v["session"]["pages"], json!(reversed));
    assert_eq!(v["session"]["hintTags"], json!(["Privat"]));
    let (st, _) = e
        .json(
            "PUT",
            format!("{scan}/session"),
            Some(json!({ "pages": [pages[0]] })),
        )
        .await;
    assert_eq!(st, Status::BadRequest);
    let (ct, body) = multipart(&[("foto.jpg", PAGE)]);
    let res = e
        .client
        .post(format!("{scan}/photos"))
        .header(ct)
        .header(rocket::http::Accept::JSON)
        .body(body)
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Ok);
    let v: Value = res.into_json().await.unwrap();
    let photo = v["session"]["pages"][2].as_str().unwrap().to_string();
    let (ct, body) = multipart(&[("kein.jpg", b"nur Text")]);
    let res = e
        .client
        .post(format!("{scan}/photos"))
        .header(ct)
        .header(rocket::http::Accept::JSON)
        .body(body)
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::BadRequest);
    drop(res);
    let (_, v) = e
        .json("DELETE", format!("{scan}/pages/{photo}"), None)
        .await;
    assert_eq!(v["session"]["pages"].as_array().unwrap().len(), 2);

    // Zur Prüfung → KI liest die Seiten (als Bilder, mit der Tag-Liste)
    let (st, v) = e.json("POST", format!("{scan}/finish"), None).await;
    assert_eq!(st, Status::Ok, "{v}");
    let id = v["item"]["id"].as_str().unwrap().to_string();
    assert_eq!(v["item"]["status"], "analyse");
    let (_, v) = e.json("GET", format!("{scan}/session"), None).await;
    assert_eq!(v["session"]["pages"], json!([]), "Sitzung geleert");
    let item = e.wait_reviewed(&org, &id).await;
    assert_eq!(item["status"], "pruefen", "{item}");
    assert_eq!(item["name"], "2026-10-15 Swisscom Rechnung Mobile");
    assert_eq!(
        item["tags"],
        json!(["Privat", "Rechnung"]),
        "vorgegeben + erkannt, Erfundenes fällt weg"
    );
    assert_eq!(item["metadata"]["absender"], "Swisscom");
    assert!(item["metadata"].get("iban").is_none());
    assert_eq!(item["model"], "mock/vision");
    assert_eq!(item["pageCount"], 2);
    let req = e.ai_requests.lock().unwrap()[0].clone();
    let text = req.to_string();
    assert!(
        text.contains("image_url")
            && text.contains("Rechnung: Rechnungen aller Art")
            && text.contains("Privat"),
        "{text}"
    );
    assert_eq!(req["model"], "openrouter/free");

    // Prüfseite und Zähler
    let html = e
        .client
        .get(format!("/organizations/{org}/review"))
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert!(html.contains("data-review-page") && html.contains("data-review-count"));
    let page = e
        .client
        .get(format!("/organizations/{org}/review/{id}/pages/2"))
        .dispatch()
        .await;
    assert_eq!(page.content_type(), Some(ContentType::JPEG));
    drop(page);
    let pdf = e
        .client
        .get(format!("/organizations/{org}/review/{id}/pdf"))
        .dispatch()
        .await
        .into_bytes()
        .await
        .unwrap();
    assert!(pdf.starts_with(b"%PDF") && String::from_utf8_lossy(&pdf).contains("/Count 2"));

    // Ändern und freigeben – mit neuem Tag und geändertem Betrag
    let edit = json!({
        "name": "Swisscom Oktober",
        "tags": ["Rechnung", "Mobile"],
        "metadata": { "absender": "Swisscom", "betrag": "1'079.90", "datum": "2026-10-15", "faellig": "kein Datum", "unbekannt": "x" },
        "summary": "Mobile-Rechnung für Oktober."
    });
    let (st, v) = e
        .json(
            "POST",
            format!("/organizations/{org}/review/{id}/approve"),
            Some(edit),
        )
        .await;
    assert_eq!(st, Status::Ok, "{v}");
    let doc = v["documentId"].as_str().unwrap().to_string();
    assert_eq!(v["url"], format!("/organizations/{org}/documents/{doc}"));

    let (_, d) = e
        .json(
            "GET",
            format!("/api/v1/organizations/{org}/documents/{doc}"),
            None,
        )
        .await;
    assert_eq!(d["document"]["name"], "Swisscom Oktober.pdf");
    let tag_names: Vec<&str> = d["document"]["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        tag_names,
        ["Mobile", "Rechnung"],
        "fehlender Tag wurde angelegt"
    );
    assert_eq!(
        d["document"]["ocrStatus"], "pending",
        "Texterkennung ist eingereiht"
    );

    let props = nrzpaper::services::properties::for_document(e.pool(), &doc)
        .await
        .unwrap();
    let shown: Vec<(String, String)> = props
        .iter()
        .map(|p| (p.name.clone(), p.value.clone()))
        .collect();
    assert_eq!(
        shown,
        [
            ("Absender".to_string(), "Swisscom".to_string()),
            ("Datum".into(), "2026-10-15".into()),
            ("Betrag".into(), "1079.9".into()),
            (
                "Zusammenfassung".into(),
                "Mobile-Rechnung für Oktober.".into()
            ),
        ]
    );
    let item = e.wait_reviewed(&org, &id).await;
    assert_eq!(item["status"], "fertig");
    assert_eq!(item["documentId"], doc.as_str());
    assert!(
        item["warning"].as_str().unwrap().contains("Fällig am"),
        "ungültiges Datum als Hinweis: {item}"
    );
    let files: i64 = sqlx::query_scalar("SELECT count(*) FROM inbox_files")
        .fetch_one(e.pool())
        .await
        .unwrap();
    assert_eq!(files, 0, "PDF und Seiten nach der Ablage gelöscht");

    // Dokumentseite zeigt Eigenschaften und Verlauf; Eigenschaft ändern
    let html = e
        .client
        .get(format!("/organizations/{org}/documents/{doc}"))
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert!(
        html.contains("Eigenschaften")
            && html.contains("Swisscom")
            && html.contains("Gescannt und geprüft (KI: mock/vision)")
    );
    let betrag = props
        .iter()
        .find(|p| p.name == "Betrag")
        .unwrap()
        .definition_id
        .clone();
    let prop_url = format!("/organizations/{org}/documents/{doc}/properties/{betrag}");
    assert_eq!(
        e.form(prop_url.clone(), "value=abc").await,
        Status::BadRequest
    );
    assert_eq!(
        e.form(prop_url.clone(), "value=12.5").await,
        Status::SeeOther
    );
    assert_eq!(e.form(prop_url, "value=").await, Status::SeeOther);
    let props = nrzpaper::services::properties::for_document(e.pool(), &doc)
        .await
        .unwrap();
    assert!(
        !props.iter().any(|p| p.name == "Betrag"),
        "leer entfernt den Wert"
    );

    // Abgelegtes lässt sich nicht noch einmal freigeben
    let (st, _) = e
        .json(
            "POST",
            format!("/organizations/{org}/review/{id}/approve"),
            Some(json!({})),
        )
        .await;
    assert_eq!(st, Status::Conflict);
    e.client.terminate().await;
}

#[tokio::test]
async fn import_failures_and_moving() {
    let e = env().await;
    let org = e.org.clone();
    let res = e
        .client
        .post("/organizations/new")
        .header(ContentType::Form)
        .body("name=Gewerbe")
        .dispatch()
        .await;
    let other = res
        .headers()
        .get_one("Location")
        .unwrap()
        .trim_start_matches("/organizations/")
        .to_string();
    drop(res);

    // KI nicht erreichbar → trotzdem prüfbar, mit Hinweis; Dateiname bleibt
    *e.ai_answer.lock().unwrap() = None;
    let (ct, body) = multipart(&[("Brief vom Amt.pdf", TEXT_PDF)]);
    let res = e
        .client
        .post(format!("/organizations/{org}/review/import"))
        .header(ct)
        .header(rocket::http::Accept::JSON)
        .body(body)
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Ok);
    let v: Value = res.into_json().await.unwrap();
    let id = v["items"][0]["id"].as_str().unwrap().to_string();
    let item = e.wait_reviewed(&org, &id).await;
    assert_eq!(item["status"], "pruefen");
    assert_eq!(item["name"], "Brief vom Amt");
    assert!(
        item["warning"].as_str().unwrap().contains("rate limited"),
        "{item}"
    );
    assert_eq!(item["pageCount"], 0);
    let sent = e.ai_requests.lock().unwrap()[0].to_string();
    assert!(
        sent.contains("application/pdf"),
        "PDF geht als Datei an die KI"
    );
    assert_eq!(
        e.ai_requests.lock().unwrap().len(),
        5,
        "openrouter/free zweimal, dann drei Bild-Modelle"
    );

    // Kein PDF → abgelehnt
    let (ct, body) = multipart(&[("bild.jpg", PAGE)]);
    let res = e
        .client
        .post(format!("/organizations/{org}/review/import"))
        .header(ct)
        .header(rocket::http::Accept::JSON)
        .body(body)
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::BadRequest);
    drop(res);

    // Neu analysieren, jetzt mit KI: der Dateiname (auto) wird ersetzt
    *e.ai_answer.lock().unwrap() = Some(swisscom());
    let (st, _) = e
        .json(
            "POST",
            format!("/organizations/{org}/review/{id}/analyze"),
            None,
        )
        .await;
    assert_eq!(st, Status::Ok);
    let item = e.wait_reviewed(&org, &id).await;
    assert_eq!(item["name"], "2026-10-15 Swisscom Rechnung Mobile");
    assert_eq!(item["warning"], "");

    // In eine fremde Organisation verschieben geht nicht, in eine eigene schon
    let (st, _) = e
        .json(
            "PUT",
            format!("/organizations/{org}/review/{id}"),
            Some(json!({ "organizationId": "gibt-es-nicht" })),
        )
        .await;
    assert_eq!(st, Status::BadRequest);
    let (st, v) = e
        .json(
            "PUT",
            format!("/organizations/{org}/review/{id}"),
            Some(json!({ "organizationId": other, "tags": ["X"] })),
        )
        .await;
    assert_eq!(st, Status::Ok);
    assert_eq!(v["item"]["organizationId"], other.as_str());
    let (_, v) = e
        .json("GET", format!("/organizations/{org}/review/items"), None)
        .await;
    assert_eq!(v["items"], json!([]));
    let (st, _) = e
        .json("GET", format!("/organizations/{org}/review/{id}/pdf"), None)
        .await;
    assert_eq!(st, Status::NotFound);

    // Duplikat: dasselbe PDF liegt schon als Dokument → Fehler, Eintrag bleibt
    let (ct, body) = multipart(&[("schon-da.pdf", TEXT_PDF)]);
    let res = e
        .client
        .post(format!("/organizations/{other}/documents"))
        .header(ct)
        .header(rocket::http::Accept::JSON)
        .body(body)
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Created);
    drop(res);
    let (st, v) = e
        .json(
            "POST",
            format!("/organizations/{other}/review/{id}/approve"),
            Some(json!({})),
        )
        .await;
    assert_eq!(st, Status::Conflict, "{v}");
    let item = e.wait_reviewed(&other, &id).await;
    assert_eq!(item["status"], "fehler");
    assert!(item["error"].as_str().unwrap().contains("gibt es schon"));

    // Verwerfen
    let (st, _) = e
        .json(
            "DELETE",
            format!("/organizations/{other}/review/{id}"),
            None,
        )
        .await;
    assert_eq!(st, Status::Ok);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM inbox_items")
        .fetch_one(e.pool())
        .await
        .unwrap();
    assert_eq!(n, 0);
    e.client.terminate().await;
}

#[tokio::test]
async fn scanner_settings_and_access() {
    let e = env().await;
    let org = e.org.clone();
    // Konto: Scanner und KI-Bereich
    let html = e
        .client
        .get(format!("/organizations/{org}/account"))
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert!(html.contains("data-account-scan") && html.contains("openrouter/free"));
    assert_eq!(
        e.form(
            "/scanner/settings".into(),
            "scanner_url=192.168.0.70/&dpi=600&next=/x"
        )
        .await,
        Status::SeeOther
    );
    let s: (String, bool, i64) =
        sqlx::query_as("SELECT scanner_url, color, dpi FROM scan_settings")
            .fetch_one(e.pool())
            .await
            .unwrap();
    assert_eq!(s, ("http://192.168.0.70".to_string(), false, 600));
    assert_eq!(
        e.form("/scanner/settings".into(), "scanner_url=http://&dpi=300")
            .await,
        Status::BadRequest
    );
    // Leer: kein Scanner → Scannen meldet das verständlich
    assert_eq!(
        e.form(
            "/scanner/settings".into(),
            "scanner_url=&color=true&dpi=300"
        )
        .await,
        Status::SeeOther
    );
    let (st, v) = e
        .json(
            "POST",
            format!("/organizations/{org}/scan/page"),
            Some(json!({})),
        )
        .await;
    assert_eq!(st, Status::BadRequest);
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Kein Scanner"));
    let (st, _) = e
        .json("POST", format!("/organizations/{org}/scan/finish"), None)
        .await;
    assert_eq!(st, Status::BadRequest, "ohne Seiten nichts zu prüfen");

    // KI-Test über das Konto
    let (st, v) = e.json("POST", "/ai/test".into(), None).await;
    assert_eq!(st, Status::Ok, "{v}");
    assert_eq!(v["model"], "mock/vision");

    // Fremde Organisation: nichts sichtbar; ohne Anmeldung: kein Zugriff
    let (st, _) = e
        .json("GET", "/organizations/fremd/review/items".into(), None)
        .await;
    assert_eq!(st, Status::NotFound);
    e.client.post("/logout").dispatch().await;
    let (st, _) = e
        .json("GET", format!("/organizations/{org}/scan/session"), None)
        .await;
    assert_eq!(st, Status::Unauthorized);
    e.client.terminate().await;
}

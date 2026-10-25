//! KI-Anbieter: Anthropic (Messages-API, eigene Kopfzeilen), lokaler OpenAI-kompatibler
//! Server ohne Schlüssel, Modell-Liste und Einstellungen – gegen Schein-Server.

mod common;

use common::call;
use rocket::http::Status;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Eine aufgezeichnete Anfrage: Methode, Pfad, Kopfzeilen (klein geschrieben), Body.
#[derive(Clone, Debug)]
struct Seen {
    method: String,
    path: String,
    head: String,
    body: Value,
}

/// Schein-KI: zeichnet Anfragen auf und antwortet je nach Pfad.
async fn fake(answer: fn(&str) -> Value, log: Arc<Mutex<Vec<Seen>>>) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let log = log.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 16384];
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
                let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
                let mut first = head.lines().next().unwrap_or("").split(' ');
                let (method, path) = (first.next().unwrap_or("").to_string(), first.next().unwrap_or("").to_string());
                let body: Value = serde_json::from_slice(&buf[head_end..]).unwrap_or(Value::Null);
                let reply = answer(&path).to_string();
                log.lock().unwrap().push(Seen { method, path, head: head.to_lowercase(), body });
                let out = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{reply}",
                    reply.len()
                );
                let _ = sock.write_all(out.as_bytes()).await;
            });
        }
    });
    url
}

const RESULT: &str = r#"{"title":"2026-10-15 Swisscom Rechnung","tags":["Test"],"metadata":{"betrag":"79.90"},"summary":"Rechnung"}"#;

fn anthropic(path: &str) -> Value {
    if path.ends_with("/models") {
        return json!({ "data": [{ "id": "claude-haiku-5-5", "display_name": "Claude Haiku 5.5" }, { "id": "claude-opus-5-5", "display_name": "Claude Opus 5.5" }] });
    }
    json!({ "model": "claude-haiku-5-5", "content": [{ "type": "text", "text": RESULT }] })
}

fn openai(path: &str) -> Value {
    if path.ends_with("/models") {
        return json!({ "object": "list", "data": [{ "id": "llava" }, { "id": "qwen2.5vl" }] });
    }
    json!({ "model": "llava", "choices": [{ "message": { "content": RESULT } }] })
}

#[rocket::async_test]
async fn anthropic_and_local_openai_compatible() {
    let (_d, c, _org) = common::client().await;
    let log = Arc::new(Mutex::new(Vec::new()));

    // Anthropic: Schlüssel je Anbieter, eigene Adresse (Schein-Server)
    let url = fake(anthropic, log.clone()).await;
    let (s, v) = call(
        &c,
        "PUT",
        "/api/v1/settings/ai",
        Some(json!({ "disabled": false, "provider": "anthropic", "key": "sk-ant-test-0123456789", "url": format!("{url}/v1"), "model": "", "fallbacks": [] })),
    )
    .await;
    assert_eq!(s, Status::Ok, "{v}");
    assert_eq!(v["provider"], "anthropic");
    assert_eq!(v["effectiveModel"], "claude-haiku-5-5");
    assert_eq!(v["keySource"], "settings");
    assert_eq!(v["available"], true);
    assert!(!v.to_string().contains("sk-ant-test-0123456789"), "Schlüssel bleibt geheim");
    let anth = v["providers"].as_array().unwrap().iter().find(|p| p["id"] == "anthropic").unwrap().clone();
    assert_eq!((anth["hasKey"].clone(), anth["protocol"].clone()), (json!(true), json!("anthropic")));

    let (s, v) = call(&c, "POST", "/api/v1/ai/test", None).await;
    assert_eq!(s, Status::Ok, "{v}");
    assert_eq!(v["model"], "claude-haiku-5-5");
    assert_eq!(v["title"], "2026-10-15 Swisscom Rechnung");
    let seen = log.lock().unwrap().last().cloned().unwrap();
    assert_eq!((seen.method.as_str(), seen.path.as_str()), ("POST", "/v1/messages"));
    assert!(seen.head.contains("x-api-key: sk-ant-test-0123456789") && seen.head.contains("anthropic-version: 2023-06-01"));
    assert!(!seen.head.contains("authorization:"));
    assert_eq!(seen.body["messages"][0]["content"][1]["type"], "image");
    assert!(seen.body["system"].as_str().unwrap().contains("title"));

    let (_, v) = call(&c, "GET", "/api/v1/settings/ai/models", None).await;
    assert_eq!(v["models"][0]["name"], "Claude Haiku 5.5");
    assert!(log.lock().unwrap().iter().any(|s| s.method == "GET" && s.path == "/v1/models" && s.head.contains("x-api-key")));

    // Lokaler Server (Ollama): ohne Schlüssel, ohne Authorization-Kopfzeile
    let url = fake(openai, log.clone()).await;
    let (s, v) = call(
        &c,
        "PUT",
        "/api/v1/settings/ai",
        Some(json!({ "disabled": false, "provider": "ollama", "url": format!("{url}/v1"), "model": "llava", "fallbacks": [] })),
    )
    .await;
    assert_eq!(s, Status::Ok, "{v}");
    assert_eq!((v["available"].clone(), v["keySource"].clone()), (json!(true), json!("none")));
    let (s, v) = call(&c, "POST", "/api/v1/ai/test", None).await;
    assert_eq!((s, v["model"].clone()), (Status::Ok, json!("llava")), "{v}");
    let seen = log.lock().unwrap().last().cloned().unwrap();
    assert_eq!(seen.path, "/v1/chat/completions");
    assert!(!seen.head.contains("authorization:"));
    assert!(seen.body.get("reasoning").is_none());
    assert_eq!(seen.body["messages"][1]["content"][1]["type"], "image_url");
    let (_, v) = call(&c, "GET", "/api/v1/settings/ai/models", None).await;
    assert_eq!(v["models"].as_array().unwrap().len(), 2);

    // Anthropic-Schlüssel ist beim Zurückwechseln noch da
    let (_, v) = call(
        &c,
        "PUT",
        "/api/v1/settings/ai",
        Some(json!({ "disabled": false, "provider": "anthropic", "model": "", "fallbacks": [] })),
    )
    .await;
    assert_eq!(v["keySource"], "settings");

    // Ungültiges
    for bad in [
        json!({ "disabled": false, "provider": "gibts-nicht", "model": "", "fallbacks": [] }),
        json!({ "disabled": false, "provider": "custom", "url": "ftp://x", "model": "", "fallbacks": [] }),
        json!({ "disabled": false, "provider": "openrouter", "key": "falsch-12345", "model": "", "fallbacks": [] }),
    ] {
        assert_eq!(call(&c, "PUT", "/api/v1/settings/ai", Some(bad.clone())).await.0, Status::BadRequest, "{bad}");
    }
    // Eigene API ohne Adresse und Modell: nicht bereit, mit Grund
    let (_, v) = call(
        &c,
        "PUT",
        "/api/v1/settings/ai",
        Some(json!({ "disabled": false, "provider": "custom", "url": "", "model": "", "fallbacks": [] })),
    )
    .await;
    assert_eq!(v["available"], false);
    assert!(v["missing"].as_str().unwrap().contains("Adresse"));
    c.terminate().await;
}

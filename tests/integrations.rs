//! Schritt 6: vollständige REST-API (Tags, Suche, Statistik, Verlauf, Organisation),
//! API-Keys und Passwort in der Oberfläche, Webhooks mit Signatur an einen echten
//! HTTP-Empfänger und öffentliche Freigabelinks.

mod common;

use common::{client, pool};
use nrzpaper::services::webhooks;
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// API-Key über die Konto-API anlegen; danach steht nur noch der Präfix in der Liste.
async fn create_key(client: &Client, _org: &str, permissions: &[&str]) -> String {
    let key = common::create_key(client, permissions).await;
    assert_eq!(key.len(), 43);
    let (_, v) = common::call(client, "GET", "/api/v1/me/api-keys", None).await;
    let list = v.to_string();
    assert!(!list.contains(&key));
    assert!(list.contains(&key[..9]));
    key
}

fn bearer(key: &str) -> Header<'static> {
    Header::new("Authorization", format!("Bearer {key}"))
}

fn multipart(name: &str, data: &[u8]) -> (ContentType, Vec<u8>) {
    let boundary = "nrzpaperGrenze";
    let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: application/octet-stream\r\n\r\n").into_bytes();
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (
        ContentType::new("multipart", "form-data").with_params(("boundary", boundary)),
        body,
    )
}

async fn upload(client: &Client, key: &str, org: &str, name: &str, data: &[u8]) -> Value {
    let (ct, body) = multipart(name, data);
    let res = client
        .post(format!("/api/v1/organizations/{org}/documents"))
        .header(bearer(key))
        .header(ct)
        .body(body)
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Created);
    res.into_json::<Value>().await.unwrap()["document"].clone()
}

async fn api(
    client: &Client,
    key: &str,
    method: &str,
    uri: String,
    body: Option<Value>,
) -> (Status, Value) {
    let req = match method {
        "GET" => client.get(uri),
        "POST" => client.post(uri),
        "PUT" => client.put(uri),
        "PATCH" => client.patch(uri),
        "DELETE" => client.delete(uri),
        _ => unreachable!(),
    };
    let req = req.header(bearer(key));
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

const ALL: &[&str] = &[
    "organizations:read",
    "organizations:update",
    "organizations:delete",
    "documents:read",
    "documents:create",
    "documents:update",
    "documents:delete",
    "tags:read",
    "tags:create",
    "tags:update",
    "tags:delete",
];

#[tokio::test]
async fn api_tags_search_statistics_activity() {
    let (_d, client, org) = client().await;
    let key = create_key(&client, &org, ALL).await;
    let base = format!("/api/v1/organizations/{org}");

    let doc = upload(
        &client,
        &key,
        &org,
        "rechnung.txt",
        "Stromrechnung Juni".as_bytes(),
    )
    .await;
    assert_eq!(doc["ocrStatus"], "done");
    let id = doc["id"].as_str().unwrap().to_string();
    upload(&client, &key, &org, "brief.txt", "Hallo Welt".as_bytes()).await;

    // Tags anlegen, ändern, an Dokument hängen
    let (s, v) = api(
        &client,
        &key,
        "POST",
        format!("{base}/tags"),
        Some(json!({ "name": "Steuern" })),
    )
    .await;
    assert_eq!(s, Status::Created);
    let tag = v["tag"]["id"].as_str().unwrap().to_string();
    let (s, _) = api(
        &client,
        &key,
        "POST",
        format!("{base}/tags"),
        Some(json!({ "name": "steuern" })),
    )
    .await;
    assert_eq!(s, Status::Conflict);
    let (s, v) = api(
        &client,
        &key,
        "PUT",
        format!("{base}/tags/{tag}"),
        Some(json!({ "name": "Steuern 2026", "description": "Belege" })),
    )
    .await;
    assert_eq!(s, Status::Ok);
    assert_eq!(v["tag"]["description"], "Belege");
    let (s, _) = api(
        &client,
        &key,
        "POST",
        format!("{base}/documents/{id}/tags"),
        Some(json!({ "tagId": tag })),
    )
    .await;
    assert_eq!(s, Status::Created);
    let (s, _) = api(
        &client,
        &key,
        "POST",
        format!("{base}/documents/{id}/tags"),
        Some(json!({ "tagId": tag })),
    )
    .await;
    assert_eq!(s, Status::NoContent, "zweites Mal: hing schon");

    let (_, v) = api(&client, &key, "GET", format!("{base}/tags"), None).await;
    assert_eq!(v["tags"][0]["documentsCount"], 1);
    let (_, v) = api(&client, &key, "GET", format!("{base}/documents/{id}"), None).await;
    assert_eq!(v["document"]["tags"][0]["name"], "Steuern 2026");

    // Suche über die API
    let (_, v) = api(
        &client,
        &key,
        "GET",
        format!("{base}/documents?searchQuery=strom"),
        None,
    )
    .await;
    assert_eq!(v["documentsCount"], 1);
    assert_eq!(v["documents"][0]["name"], "rechnung.txt");
    assert_eq!(v["documents"][0]["tags"][0]["id"], tag.as_str());
    let (_, v) = api(
        &client,
        &key,
        "GET",
        format!("{base}/documents?searchQuery=-tag%3A%22Steuern%202026%22"),
        None,
    )
    .await;
    assert_eq!(v["documentsCount"], 1);
    assert_eq!(v["documents"][0]["name"], "brief.txt");
    let (_, v) = api(&client, &key, "GET", format!("{base}/documents"), None).await;
    assert_eq!(v["documentsCount"], 2);

    // Statistik und Verlauf
    let (_, _) = api(
        &client,
        &key,
        "DELETE",
        format!("{base}/documents/{id}"),
        None,
    )
    .await;
    let (s, v) = api(
        &client,
        &key,
        "GET",
        format!("{base}/documents/statistics"),
        None,
    )
    .await;
    assert_eq!(s, Status::Ok);
    assert_eq!(v["organizationStats"]["documentsCount"], 1);
    assert_eq!(v["organizationStats"]["deletedDocumentsCount"], 1);
    assert_eq!(v["organizationStats"]["documentsSize"], 10);
    let (_, v) = api(
        &client,
        &key,
        "GET",
        format!("{base}/documents/{id}/activity"),
        None,
    )
    .await;
    let events: Vec<&str> = v["activities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["event"].as_str().unwrap())
        .collect();
    assert_eq!(events, ["deleted", "tag_added", "created"]);
    assert_eq!(v["activities"][1]["data"]["tag"], "Steuern 2026");
    assert_eq!(v["activities"][1]["username"], "alice");

    // Tag entfernen und löschen
    let (s, _) = api(
        &client,
        &key,
        "DELETE",
        format!("{base}/documents/{id}/tags/{tag}"),
        None,
    )
    .await;
    assert_eq!(s, Status::NoContent);
    let (s, _) = api(&client, &key, "DELETE", format!("{base}/tags/{tag}"), None).await;
    assert_eq!(s, Status::NoContent);
    let (_, v) = api(&client, &key, "GET", format!("{base}/tags"), None).await;
    assert_eq!(v["tags"], json!([]));

    // Organisation umbenennen und löschen
    let (s, v) = api(
        &client,
        &key,
        "PUT",
        format!("/api/v1/organizations/{org}"),
        Some(json!({ "name": "Zuhause" })),
    )
    .await;
    assert_eq!(s, Status::Ok);
    assert_eq!(v["organization"]["name"], "Zuhause");
    let (s, _) = api(
        &client,
        &key,
        "DELETE",
        format!("/api/v1/organizations/{org}"),
        None,
    )
    .await;
    assert_eq!(s, Status::NoContent);
    let (s, _) = api(
        &client,
        &key,
        "GET",
        format!("/api/v1/organizations/{org}"),
        None,
    )
    .await;
    assert_eq!(s, Status::NotFound);

    client.terminate().await;
}

#[tokio::test]
async fn api_key_permissions_are_enforced() {
    let (_d, client, org) = client().await;
    let key = create_key(&client, &org, &["documents:read", "tags:read"]).await;
    let base = format!("/api/v1/organizations/{org}");
    let (s, _) = api(&client, &key, "GET", format!("{base}/tags"), None).await;
    assert_eq!(s, Status::Ok);
    let (s, v) = api(
        &client,
        &key,
        "POST",
        format!("{base}/tags"),
        Some(json!({ "name": "x" })),
    )
    .await;
    assert_eq!(s, Status::Forbidden, "{v}");
    let (s, _) = api(
        &client,
        &key,
        "PUT",
        format!("/api/v1/organizations/{org}"),
        Some(json!({ "name": "x" })),
    )
    .await;
    assert_eq!(s, Status::Forbidden);
    let (s, _) = api(&client, &key, "GET", format!("{base}/webhooks"), None).await;
    assert_eq!(s, Status::Forbidden);

    // Ohne Rechte kein Key; gelöschter Key gilt nicht mehr
    let (s, v) = common::call(
        &client,
        "POST",
        "/api/v1/me/api-keys",
        Some(json!({ "name": "Leer", "permissions": [] })),
    )
    .await;
    assert_eq!(s, Status::BadRequest);
    assert!(v.to_string().contains("Mindestens eine Berechtigung"));
    let id: String = sqlx::query_scalar("SELECT id FROM api_keys")
        .fetch_one(pool(&client))
        .await
        .unwrap();
    let (status, _) = common::call(&client, "DELETE", format!("/api/v1/me/api-keys/{id}"), None).await;
    assert_eq!(status, Status::NoContent);
    let res = client
        .get("/api/v1/api-keys/current")
        .header(bearer(&key))
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Unauthorized);
    drop(res);
    client.terminate().await;
}

/// Ein einziger HTTP-Request an einen kleinen Empfänger: (Kopfzeilen klein geschrieben, Body).
async fn receive_one(listener: &tokio::net::TcpListener) -> (Vec<(String, String)>, String) {
    let (mut sock, _) = listener.accept().await.unwrap();
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let (head_end, length) = loop {
        let n = sock.read(&mut chunk).await.unwrap();
        assert!(n > 0, "Verbindung zu früh beendet");
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
    while buf.len() < head_end + length {
        let n = sock.read(&mut chunk).await.unwrap();
        buf.extend_from_slice(&chunk[..n]);
    }
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let headers = head
        .lines()
        .skip(1)
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
        .collect();
    let body = String::from_utf8_lossy(&buf[head_end..head_end + length]).to_string();
    sock.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nok")
        .await
        .unwrap();
    (headers, body)
}

#[tokio::test]
async fn webhooks_deliver_signed_events() {
    let (_d, client, org) = client().await;
    let key = create_key(&client, &org, ALL).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/hook", listener.local_addr().unwrap());
    let base = format!("/api/v1/organizations/{org}");

    // Ungültige URL und leere Ereignisliste werden abgelehnt
    let (s, _) = api(
        &client,
        &key,
        "POST",
        format!("{base}/webhooks"),
        Some(json!({ "name": "x", "url": "ftp://x", "events": ["document:created"] })),
    )
    .await;
    assert_eq!(s, Status::BadRequest);
    let (s, _) = api(
        &client,
        &key,
        "POST",
        format!("{base}/webhooks"),
        Some(json!({ "name": "x", "url": url, "events": [] })),
    )
    .await;
    assert_eq!(s, Status::BadRequest);

    let (s, v) = api(
        &client,
        &key,
        "POST",
        format!("{base}/webhooks"),
        Some(json!({ "name": "Empfänger", "url": url, "secret": "whsec_test", "events": ["document:created", "document:tag:added"] })),
    )
    .await;
    assert_eq!(s, Status::Created, "{v}");
    assert_eq!(v["secret"], "whsec_test");
    assert!(
        v["webhook"].get("secret").is_none(),
        "Geheimnis nie in Listen"
    );
    let hook = v["webhook"]["id"].as_str().unwrap().to_string();

    let doc = upload(&client, &key, &org, "notiz.txt", b"Webhook-Test").await;
    let (headers, body) = tokio::time::timeout(Duration::from_secs(10), receive_one(&listener))
        .await
        .expect("Webhook kam nicht an");
    let get = |k: &str| {
        headers
            .iter()
            .find(|(h, _)| h == k)
            .map(|(_, v)| v.clone())
            .unwrap()
    };
    let id = get("webhook-id");
    let ts: i64 = get("webhook-timestamp").parse().unwrap();
    assert!(id.starts_with("msg_"));
    assert_eq!(get("content-type"), "application/json");
    assert_eq!(
        get("webhook-signature"),
        webhooks::sign("whsec_test", &id, ts, &body),
        "Signatur stimmt"
    );
    let payload: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(payload["type"], "document:created");
    assert_eq!(payload["data"]["documentId"], doc["id"]);
    assert_eq!(payload["data"]["name"], "notiz.txt");

    // Zustellung steht im Protokoll (wird nach der Antwort geschrieben)
    let mut logged = Value::Null;
    for _ in 0..50 {
        let (_, v) = api(
            &client,
            &key,
            "GET",
            format!("{base}/webhooks/{hook}/deliveries"),
            None,
        )
        .await;
        if !v["deliveries"].as_array().unwrap().is_empty() {
            logged = v;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(logged["deliveries"][0]["responseStatus"], 200);
    assert_eq!(logged["deliveries"][0]["event"], "document:created");

    // Pausiert: kein Versand
    let (s, _) = api(
        &client,
        &key,
        "PATCH",
        format!("{base}/webhooks/{hook}"),
        Some(json!({ "enabled": false })),
    )
    .await;
    assert_eq!(s, Status::NoContent);
    upload(&client, &key, &org, "zwei.txt", b"noch einer").await;
    assert!(
        tokio::time::timeout(Duration::from_millis(700), listener.accept())
            .await
            .is_err()
    );

    // Oberfläche (Sitzung): Liste ohne Geheimnis, ein neues wird genau einmal gezeigt
    let (_, v) = common::call(&client, "GET", format!("{base}/webhooks"), None).await;
    let list = v.to_string();
    assert!(list.contains("Empfänger") && !list.contains("whsec_test"));
    let (s, v) = common::call(
        &client,
        "POST",
        format!("{base}/webhooks"),
        Some(json!({ "name": "Zweiter", "url": url, "events": ["document:deleted"] })),
    )
    .await;
    assert_eq!(s, Status::Created);
    assert!(
        v["secret"].as_str().unwrap().starts_with("whsec_"),
        "erzeugtes Geheimnis wird gezeigt"
    );

    let (s, _) = api(
        &client,
        &key,
        "DELETE",
        format!("{base}/webhooks/{hook}"),
        None,
    )
    .await;
    assert_eq!(s, Status::NoContent);
    let (_, v) = api(&client, &key, "GET", format!("{base}/webhooks"), None).await;
    assert_eq!(v["webhooks"].as_array().unwrap().len(), 1);
    client.terminate().await;
}

/// Status der öffentlichen Freigabe (JSON der Oberfläche).
async fn shared(client: &Client, token: &str) -> Status {
    common::call(client, "GET", format!("/api/v1/share/{token}"), None).await.0
}

#[tokio::test]
async fn share_links_are_public_until_revoked() {
    let (_d, client, org) = client().await;
    let key = create_key(&client, &org, ALL).await;
    let doc = upload(
        &client,
        &key,
        &org,
        "vertrag.txt",
        b"Mietvertrag Seestrasse",
    )
    .await;
    let id = doc["id"].as_str().unwrap().to_string();
    let detail = format!("/organizations/{org}/documents/{id}");

    // In der Oberfläche (Sitzung) anlegen: der Link steht nur in dieser Antwort
    let (s, v) = common::call(
        &client,
        "POST",
        format!("/api/v1{detail}/share-links"),
        Some(json!({ "expiresInDays": 7 })),
    )
    .await;
    assert_eq!(s, Status::Created);
    let token = v["token"].as_str().unwrap().to_string();
    assert!(token.starts_with("sh_"));
    let (_, v) = common::call(&client, "GET", format!("/api/v1{detail}/share-links"), None).await;
    assert!(!v.to_string().contains(&token));

    // Über die API ein zweiter, unbefristeter Link
    let (s, v) = api(
        &client,
        &key,
        "POST",
        format!("/api/v1{detail}/share-links"),
        None,
    )
    .await;
    assert_eq!(s, Status::Created);
    let token2 = v["token"].as_str().unwrap().to_string();
    let link2 = v["shareLink"]["id"].as_str().unwrap().to_string();
    assert_eq!(v["path"], format!("/share/{token2}"));
    assert!(v["shareLink"]["expiresAt"].is_null());

    // Abmelden: Freigabe funktioniert ohne Konto
    common::logout(&client).await;
    let res = client.get(format!("/share/{token}")).dispatch().await;
    assert_eq!(res.status(), Status::Ok, "Seite der Oberfläche");
    assert_eq!(res.headers().get_one("X-Robots-Tag"), Some("noindex"));
    drop(res);
    let (s, v) = common::call(&client, "GET", format!("/api/v1/share/{token}"), None).await;
    assert_eq!(s, Status::Ok);
    assert_eq!(v["document"]["name"], "vertrag.txt");
    assert_eq!(v["document"]["content"], "Mietvertrag Seestrasse");
    assert_eq!(v["fileUrl"], format!("/share/{token}/file"));
    let res = client
        .get(format!("/share/{token}/file"))
        .header(Header::new("Range", "bytes=0-3"))
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::PartialContent);
    assert_eq!(res.into_bytes().await.unwrap(), b"Miet");
    let res = client
        .get(format!("/share/{token}/file?download=true"))
        .dispatch()
        .await;
    assert!(res
        .headers()
        .get_one("Content-Disposition")
        .unwrap()
        .starts_with("attachment"));
    drop(res);
    // Die Organisation bleibt ohne Anmeldung zu
    assert_eq!(
        common::call(&client, "GET", format!("/api/v1{detail}"), None).await.0,
        Status::Unauthorized
    );
    for bad in ["sh_falsch", "np_abc", "x"] {
        assert_eq!(shared(&client, bad).await, Status::NotFound);
    }

    // Aufrufe gezählt (nur die Seite, nicht die Datei)
    let (_, v) = api(
        &client,
        &key,
        "GET",
        format!("/api/v1{detail}/share-links"),
        None,
    )
    .await;
    let links = v["shareLinks"].as_array().unwrap();
    assert_eq!(links.len(), 2);
    let first = links.iter().find(|l| l["id"] != link2.as_str()).unwrap();
    assert_eq!(first["viewCount"], 1);
    assert!(first.get("tokenHash").is_none());

    // Abgelaufen → 404
    sqlx::query("UPDATE share_links SET expires_at = 1 WHERE id = ?")
        .bind(first["id"].as_str().unwrap())
        .execute(pool(&client))
        .await
        .unwrap();
    assert_eq!(
        shared(&client, &token).await,
        Status::NotFound
    );

    // Papierkorb → 404, wiederhergestellt → wieder da
    assert_eq!(
        shared(&client, &token2).await,
        Status::Ok
    );
    api(&client, &key, "DELETE", format!("/api/v1{detail}"), None).await;
    assert_eq!(
        client
            .get(format!("/share/{token2}/file"))
            .dispatch()
            .await
            .status(),
        Status::NotFound
    );
    let (s, _) = api(
        &client,
        &key,
        "POST",
        format!("/api/v1{detail}/share-links"),
        None,
    )
    .await;
    assert_eq!(s, Status::BadRequest, "nichts aus dem Papierkorb freigeben");
    api(
        &client,
        &key,
        "POST",
        format!("/api/v1{detail}/restore"),
        None,
    )
    .await;
    assert_eq!(
        shared(&client, &token2).await,
        Status::Ok
    );

    // Widerrufen → 404, zweites Widerrufen → 404
    let (s, _) = api(
        &client,
        &key,
        "DELETE",
        format!("/api/v1{detail}/share-links/{link2}"),
        None,
    )
    .await;
    assert_eq!(s, Status::NoContent);
    assert_eq!(
        shared(&client, &token2).await,
        Status::NotFound
    );
    let (s, _) = api(
        &client,
        &key,
        "DELETE",
        format!("/api/v1{detail}/share-links/{link2}"),
        None,
    )
    .await;
    assert_eq!(s, Status::NotFound);
    client.terminate().await;
}

#[tokio::test]
async fn password_change_ends_other_sessions() {
    let (_d, client, _org) = client().await;
    let sessions = || async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM sessions")
            .fetch_one(pool(&client))
            .await
            .unwrap()
    };
    // Zweite Anmeldung (anderes Gerät)
    assert_eq!(common::login(&client, "alice", "geheim-123").await, Status::Ok);
    assert_eq!(sessions().await, 2);

    let change = |current: &str, new: &str, confirm: &str| {
        let body = json!({ "current": current, "new": new, "confirm": confirm });
        let c = &client;
        async move { common::call(c, "PUT", "/api/v1/me/password", Some(body)).await }
    };
    let (s, v) = change("falsch", "neues-pass-1", "neues-pass-1").await;
    assert_eq!(s, Status::BadRequest);
    assert!(v.to_string().contains("aktuelle Passwort stimmt nicht"));
    assert_eq!(change("geheim-123", "neues-pass-1", "anders-pass-1").await.0, Status::BadRequest);
    assert_eq!(change("geheim-123", "kurz", "kurz").await.0, Status::BadRequest);
    assert_eq!(sessions().await, 2, "Fehlversuche ändern nichts");

    assert_eq!(change("geheim-123", "neues-pass-1", "neues-pass-1").await.0, Status::Ok);
    assert_eq!(sessions().await, 1, "nur noch diese Sitzung");
    assert_eq!(common::call(&client, "GET", "/api/v1/me", None).await.0, Status::Ok);

    common::logout(&client).await;
    assert_eq!(
        common::login(&client, "alice", "geheim-123").await,
        Status::Unauthorized,
        "altes Passwort gilt nicht mehr"
    );
    assert_eq!(common::login(&client, "alice", "neues-pass-1").await, Status::Ok);
    client.terminate().await;
}

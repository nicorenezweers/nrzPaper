//! JSON-API für die Dioxus-Oberfläche: Anmeldung, Konto, Einstellungen (KI/OpenRouter,
//! Registrierung), Tagging-Regeln, Eigenschaften, Ordner, Papierkorb.

use rocket::http::{Accept, ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use serde_json::{json, Value};
use sqlx::SqlitePool;

async fn client() -> (tempfile::TempDir, Client) {
    let dir = tempfile::tempdir().unwrap();
    let figment = nrzpaper::config::figment()
        .merge(("data_dir", dir.path()))
        .merge(("db_key", "test"))
        .merge(("log_level", "off"))
        .merge(("background", false))
        .merge(("openrouter_url", "http://127.0.0.1:9"));
    (
        dir,
        Client::tracked(nrzpaper::build_with(figment).await.unwrap())
            .await
            .unwrap(),
    )
}

async fn call(c: &Client, method: &str, uri: &str, body: Option<Value>) -> (Status, Value) {
    let req = match method {
        "GET" => c.get(uri.to_string()),
        "POST" => c.post(uri.to_string()),
        "PUT" => c.put(uri.to_string()),
        "DELETE" => c.delete(uri.to_string()),
        _ => unreachable!(),
    }
    .header(Accept::JSON);
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

fn pool(c: &Client) -> &SqlitePool {
    c.rocket().state::<SqlitePool>().unwrap()
}

/// Erstes Konto (Admin) und eine Organisation.
async fn setup(c: &Client) -> String {
    let (s, _) = call(
        c,
        "POST",
        "/api/v1/auth/register",
        Some(json!({ "username": "alice", "password": "geheim-123" })),
    )
    .await;
    assert_eq!(s, Status::Created);
    let (_, v) = call(
        c,
        "POST",
        "/api/v1/organizations",
        Some(json!({ "name": "Privat" })),
    )
    .await;
    v["organization"]["id"].as_str().unwrap().to_string()
}

async fn upload_text(c: &Client, org: &str, name: &str, text: &str) -> String {
    let body = format!("--G\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: text/plain\r\n\r\n{text}\r\n--G--\r\n");
    let res = c
        .post(format!("/api/v1/organizations/{org}/documents"))
        .header(ContentType::new("multipart", "form-data").with_params(("boundary", "G")))
        .header(Accept::JSON)
        .body(body)
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Created);
    let v: Value = res.into_json().await.unwrap();
    v["document"]["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn auth_and_account() {
    let (_d, c) = client().await;
    let (_, v) = call(&c, "GET", "/api/v1/auth/status", None).await;
    assert_eq!(v["needsSetup"], true);
    assert_eq!(v["loggedIn"], false);
    let org = setup(&c).await;
    let (_, v) = call(&c, "GET", "/api/v1/me", None).await;
    assert_eq!(v["user"]["isAdmin"], true);
    assert_eq!(v["organizations"][0]["id"], org.as_str());

    // Abmelden, falsch und richtig anmelden
    call(&c, "POST", "/api/v1/auth/logout", None).await;
    let (s, _) = call(&c, "GET", "/api/v1/me", None).await;
    assert_eq!(s, Status::Unauthorized);
    let (s, _) = call(
        &c,
        "POST",
        "/api/v1/auth/login",
        Some(json!({ "username": "alice", "password": "falsch" })),
    )
    .await;
    assert_eq!(s, Status::Unauthorized);
    let (s, _) = call(
        &c,
        "POST",
        "/api/v1/auth/login",
        Some(json!({ "username": "alice", "password": "geheim-123" })),
    )
    .await;
    assert_eq!(s, Status::Ok);
    // Registrierung danach geschlossen
    let (_, v) = call(&c, "GET", "/api/v1/auth/status", None).await;
    assert_eq!(
        (v["needsSetup"].clone(), v["registrationOpen"].clone()),
        (json!(false), json!(false))
    );

    // Passwort, API-Keys, Scanner
    let (s, _) = call(
        &c,
        "PUT",
        "/api/v1/me/password",
        Some(json!({ "current": "x", "new": "neues-pass-1", "confirm": "neues-pass-1" })),
    )
    .await;
    assert_eq!(s, Status::BadRequest);
    let (s, _) = call(
        &c,
        "PUT",
        "/api/v1/me/password",
        Some(json!({ "current": "geheim-123", "new": "neues-pass-1", "confirm": "neues-pass-1" })),
    )
    .await;
    assert_eq!(s, Status::Ok);
    let (s, _) = call(&c, "GET", "/api/v1/me", None).await;
    assert_eq!(s, Status::Ok, "Sitzung bleibt nach dem Passwortwechsel");
    let (s, v) = call(
        &c,
        "POST",
        "/api/v1/me/api-keys",
        Some(json!({ "name": "Skript", "permissions": ["documents:read"], "expiresInDays": 30 })),
    )
    .await;
    assert_eq!(s, Status::Created);
    let key = v["key"].as_str().unwrap().to_string();
    let (_, v) = call(&c, "GET", "/api/v1/me/api-keys", None).await;
    assert_eq!(v["apiKeys"].as_array().unwrap().len(), 1);
    assert!(!v.to_string().contains(&key));
    let (_, v) = call(
        &c,
        "PUT",
        "/api/v1/me/scanner",
        Some(json!({ "scannerUrl": "192.168.1.50", "color": false, "dpi": 150 })),
    )
    .await;
    assert_eq!(v["scanner"]["scannerUrl"], "http://192.168.1.50");

    // Fremde Seite darf nichts ändern
    let res = c
        .post("/api/v1/auth/logout")
        .header(Header::new("Origin", "https://boese.example"))
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Ok, "Abmelden ist unkritisch");
    drop(res);
    c.terminate().await;
}

#[tokio::test]
async fn ai_and_instance_settings() {
    let (_d, c) = client().await;
    let org = setup(&c).await;
    let (_, v) = call(&c, "GET", "/api/v1/settings/ai", None).await;
    assert_eq!(v["effectiveModel"], "openrouter/free");
    assert_eq!(v["model"], "");
    let (s, _) = call(
        &c,
        "PUT",
        "/api/v1/settings/ai",
        Some(json!({ "disabled": false, "model": "", "fallbacks": [], "key": "kein-schluessel" })),
    )
    .await;
    assert_eq!(s, Status::BadRequest);
    let secret = "sk-or-v1-0123456789abcdef0123456789abcdef";
    let (s, v) = call(&c, "PUT", "/api/v1/settings/ai", Some(json!({ "disabled": false, "model": "google/gemma-4-31b-it:free", "fallbacks": ["a:free", " "], "key": secret }))).await;
    assert_eq!(s, Status::Ok, "{v}");
    assert_eq!(v["keySource"], "settings");
    assert_eq!(v["effectiveModel"], "google/gemma-4-31b-it:free");
    assert_eq!(v["fallbacks"], json!(["a:free"]));
    assert!(
        !v.to_string().contains(secret),
        "Schlüssel verlässt den Server nie"
    );
    assert_eq!(v["keyHint"], "sk-or-v1-0…cdef");
    // Leerer Schlüssel behält den alten; ausschalten
    let (_, v) = call(
        &c,
        "PUT",
        "/api/v1/settings/ai",
        Some(json!({ "disabled": true, "model": "", "fallbacks": [] })),
    )
    .await;
    assert_eq!(
        (v["keySource"].clone(), v["available"].clone()),
        (json!("settings"), json!(false))
    );
    let (_, v) = call(
        &c,
        "GET",
        &format!("/api/v1/organizations/{org}/overview"),
        None,
    )
    .await;
    assert_eq!(v["aiAvailable"], false);
    let (s, v) = call(&c, "POST", "/api/v1/ai/test", None).await;
    assert_eq!(s, Status::BadRequest);
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("ausgeschaltet"));
    let (_, v) = call(
        &c,
        "PUT",
        "/api/v1/settings/ai",
        Some(json!({ "disabled": false, "model": "", "fallbacks": [], "clearKey": true })),
    )
    .await;
    assert_ne!(v["keySource"], "settings");

    // Registrierung öffnen → zweites Konto möglich, das ist kein Admin
    let (_, v) = call(
        &c,
        "PUT",
        "/api/v1/settings/instance",
        Some(json!({ "registrationOpen": true })),
    )
    .await;
    assert_eq!(v["registrationOpen"], true);
    call(&c, "POST", "/api/v1/auth/logout", None).await;
    let (s, _) = call(
        &c,
        "POST",
        "/api/v1/auth/register",
        Some(json!({ "username": "zweite", "password": "geheim-123" })),
    )
    .await;
    assert_eq!(s, Status::Created);
    let (s, _) = call(&c, "GET", "/api/v1/settings/ai", None).await;
    assert_eq!(s, Status::Forbidden, "nur Admins");
    let (s, _) = call(&c, "GET", "/api/v1/settings/users", None).await;
    assert_eq!(s, Status::Forbidden);
    c.terminate().await;
}

#[tokio::test]
async fn tagging_rules_apply() {
    let (_d, c) = client().await;
    let org = setup(&c).await;
    let base = format!("/api/v1/organizations/{org}");
    let (_, v) = call(
        &c,
        "POST",
        &format!("{base}/tags"),
        Some(json!({ "name": "Rechnung" })),
    )
    .await;
    let rechnung = v["tag"]["id"].as_str().unwrap().to_string();
    let (_, v) = call(
        &c,
        "POST",
        &format!("{base}/tags"),
        Some(json!({ "name": "Telefon" })),
    )
    .await;
    let telefon = v["tag"]["id"].as_str().unwrap().to_string();

    // Vorhandenes Dokument vor der Regel
    let old = upload_text(&c, &org, "alt.txt", "Swisscom Rechnung September").await;

    // Ungültig: kein Tag, leerer Wert, kaputter Ausdruck
    for bad in [
        json!({ "name": "x", "conditions": [{ "field": "content", "operator": "contains", "value": "a" }], "tagIds": [] }),
        json!({ "name": "x", "conditions": [{ "field": "content", "operator": "contains", "value": " " }], "tagIds": [rechnung] }),
        json!({ "name": "x", "conditions": [{ "field": "content", "operator": "regex", "value": "(" }], "tagIds": [rechnung] }),
    ] {
        let (s, _) = call(&c, "POST", &format!("{base}/tagging-rules"), Some(bad)).await;
        assert_eq!(s, Status::BadRequest);
    }
    let rule = json!({
        "name": "Swisscom-Rechnungen",
        "matchMode": "all",
        "conditions": [
            { "field": "content", "operator": "contains", "value": "swisscom" },
            { "field": "content", "operator": "regex", "value": "rechnung|faktura" }
        ],
        "tagIds": [rechnung, telefon]
    });
    let (s, v) = call(&c, "POST", &format!("{base}/tagging-rules"), Some(rule)).await;
    assert_eq!(s, Status::Created, "{v}");
    let id = v["taggingRule"]["id"].as_str().unwrap().to_string();
    assert_eq!(v["taggingRule"]["conditions"].as_array().unwrap().len(), 2);

    // Neues Text-Dokument: Regel greift sofort
    let new = upload_text(&c, &org, "neu.txt", "SWISSCOM Faktura Oktober").await;
    let other = upload_text(&c, &org, "anders.txt", "Salt Rechnung").await;
    let tag_names = |doc: String| {
        let pool = pool(&c).clone();
        async move {
            sqlx::query_scalar::<_, String>("SELECT t.name FROM document_tags dt JOIN tags t ON t.id = dt.tag_id WHERE dt.document_id = ? ORDER BY t.name")
                .bind(doc)
                .fetch_all(&pool)
                .await
                .unwrap()
        }
    };
    assert_eq!(tag_names(new.clone()).await, ["Rechnung", "Telefon"]);
    assert!(tag_names(other.clone()).await.is_empty());
    assert!(
        tag_names(old.clone()).await.is_empty(),
        "ältere Dokumente erst auf Wunsch"
    );

    // Rückwirkend anwenden
    let (_, v) = call(
        &c,
        "POST",
        &format!("{base}/tagging-rules/{id}/apply"),
        None,
    )
    .await;
    assert_eq!(v, json!({ "matched": 2, "tagsAdded": 2 }));
    assert_eq!(tag_names(old).await, ["Rechnung", "Telefon"]);

    // Ändern (any) und ausschalten
    let (s, v) = call(
        &c,
        "PUT",
        &format!("{base}/tagging-rules/{id}"),
        Some(json!({ "name": "Rechnungen", "enabled": false, "matchMode": "any", "conditions": [{ "field": "name", "operator": "ends_with", "value": ".txt" }], "tagIds": [rechnung] })),
    )
    .await;
    assert_eq!(s, Status::Ok);
    assert_eq!(v["taggingRule"]["enabled"], false);
    let later = upload_text(&c, &org, "spaeter.txt", "Swisscom Rechnung").await;
    assert!(
        tag_names(later).await.is_empty(),
        "ausgeschaltete Regel greift nicht"
    );
    let (s, _) = call(&c, "DELETE", &format!("{base}/tagging-rules/{id}"), None).await;
    assert_eq!(s, Status::NoContent);
    let (_, v) = call(&c, "GET", &format!("{base}/tagging-rules"), None).await;
    assert_eq!(v["taggingRules"], json!([]));
    c.terminate().await;
}

#[tokio::test]
async fn custom_properties_all_types() {
    let (_d, c) = client().await;
    let org = setup(&c).await;
    let base = format!("/api/v1/organizations/{org}");
    let doc = upload_text(&c, &org, "vertrag.txt", "Mietvertrag").await;
    let other = upload_text(&c, &org, "anhang.txt", "Anhang").await;

    let mk = |body: Value| {
        let c = &c;
        let base = base.clone();
        async move { call(c, "POST", &format!("{base}/custom-properties"), Some(body)).await }
    };
    let (s, _) = mk(json!({ "name": "Status", "type": "select", "options": [] })).await;
    assert_eq!(s, Status::BadRequest, "Auswahl ohne Möglichkeiten");
    let (s, v) =
        mk(json!({ "name": "Status", "type": "select", "options": ["Offen", "Bezahlt", "offen"] }))
            .await;
    assert_eq!(s, Status::Created);
    let status = v["customProperty"]["id"].as_str().unwrap().to_string();
    assert_eq!(
        v["customProperty"]["options"].as_array().unwrap().len(),
        2,
        "doppelte Möglichkeit entfernt"
    );
    let (s, _) = mk(json!({ "name": "status", "type": "text" })).await;
    assert_eq!(s, Status::Conflict);
    let ids: Vec<String> = {
        let mut out = Vec::new();
        for (name, kind, opts) in [
            ("Kategorien", "multi_select", json!(["Wohnen", "Steuern"])),
            ("Betrag", "number", json!([])),
            ("Fällig", "date", json!([])),
            ("Bezahlt", "boolean", json!([])),
            ("Zuständig", "user_relation", json!([])),
            ("Siehe auch", "document_relation", json!([])),
        ] {
            let (_, v) = mk(json!({ "name": name, "type": kind, "options": opts })).await;
            out.push(v["customProperty"]["id"].as_str().unwrap().to_string());
        }
        out
    };
    let set = |def: String, values: Value| {
        let c = &c;
        let url = format!("{base}/documents/{doc}/custom-properties/{def}");
        async move { call(c, "PUT", &url, Some(json!({ "values": values }))).await }
    };
    assert_eq!(
        set(status.clone(), json!(["bezahlt"])).await.0,
        Status::Ok,
        "Möglichkeit per Name"
    );
    assert_eq!(
        set(status.clone(), json!(["Offen", "Bezahlt"])).await.0,
        Status::BadRequest,
        "nur ein Wert"
    );
    assert_eq!(
        set(status.clone(), json!(["Unbekannt"])).await.0,
        Status::BadRequest
    );
    assert_eq!(
        set(ids[0].clone(), json!(["Wohnen", "Steuern", "Wohnen"]))
            .await
            .0,
        Status::Ok
    );
    assert_eq!(
        set(ids[1].clone(), json!(["CHF 1'250.50"])).await.0,
        Status::Ok
    );
    assert_eq!(
        set(ids[2].clone(), json!(["31.12.2026"])).await.0,
        Status::BadRequest
    );
    assert_eq!(
        set(ids[2].clone(), json!(["2026-12-31"])).await.0,
        Status::Ok
    );
    assert_eq!(set(ids[3].clone(), json!(["true"])).await.0, Status::Ok);
    assert_eq!(set(ids[4].clone(), json!(["alice"])).await.0, Status::Ok);
    assert_eq!(
        set(ids[4].clone(), json!(["fremder"])).await.0,
        Status::BadRequest
    );
    assert_eq!(
        set(ids[5].clone(), json!([doc.clone()])).await.0,
        Status::BadRequest,
        "nicht auf sich selbst"
    );
    let (s, v) = set(ids[5].clone(), json!([other.clone()])).await;
    assert_eq!(s, Status::Ok);
    let values = v["values"].as_array().unwrap();
    let get = |name: &str| values.iter().find(|x| x["name"] == name).unwrap().clone();
    assert_eq!(get("Status")["display"], json!(["Bezahlt"]));
    assert_eq!(get("Kategorien")["display"], json!(["Wohnen", "Steuern"]));
    assert_eq!(get("Betrag")["values"], json!(["1250.5"]));
    assert_eq!(get("Fällig")["values"], json!(["2026-12-31"]));
    assert_eq!(get("Bezahlt")["display"], json!(["Ja"]));
    assert_eq!(get("Zuständig")["display"], json!(["alice"]));
    assert_eq!(get("Siehe auch")["display"], json!(["anhang.txt"]));

    // Möglichkeiten ändern: bestehende behalten Werte, entfernte verlieren sie; Typ fest
    let (s, v) = call(
        &c,
        "PUT",
        &format!("{base}/custom-properties/{status}"),
        Some(json!({ "name": "Zahlstatus", "type": "select", "options": ["Bezahlt", "Gemahnt"] })),
    )
    .await;
    assert_eq!(s, Status::Ok, "{v}");
    let (s, _) = call(
        &c,
        "PUT",
        &format!("{base}/custom-properties/{status}"),
        Some(json!({ "name": "Zahlstatus", "type": "text" })),
    )
    .await;
    assert_eq!(s, Status::BadRequest);
    let (_, v) = call(
        &c,
        "GET",
        &format!("{base}/documents/{doc}/custom-properties"),
        None,
    )
    .await;
    let z = v["values"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["name"] == "Zahlstatus")
        .unwrap()
        .clone();
    assert_eq!(z["display"], json!(["Bezahlt"]));

    // Reihenfolge, leeren, löschen
    let mut order = ids.clone();
    order.insert(0, status.clone());
    order.reverse();
    let (_, v) = call(
        &c,
        "PUT",
        &format!("{base}/custom-properties/order"),
        Some(json!({ "ids": order })),
    )
    .await;
    assert_eq!(v["customProperties"][0]["name"], "Siehe auch");
    assert_eq!(set(ids[1].clone(), json!([])).await.0, Status::Ok);
    let (s, _) = call(
        &c,
        "DELETE",
        &format!("{base}/custom-properties/{status}"),
        None,
    )
    .await;
    assert_eq!(s, Status::NoContent);
    let n: i64 =
        sqlx::query_scalar("SELECT count(*) FROM document_property_values WHERE document_id = ?")
            .bind(&doc)
            .fetch_one(pool(&c))
            .await
            .unwrap();
    assert_eq!(
        n, 6,
        "Kategorien(2) + Fällig + Bezahlt + Zuständig + Siehe auch"
    );
    c.terminate().await;
}

#[tokio::test]
async fn folders_overview_and_trash() {
    let (_d, c) = client().await;
    let org = setup(&c).await;
    let base = format!("/api/v1/organizations/{org}");
    let (_, v) = call(
        &c,
        "POST",
        &format!("{base}/tags"),
        Some(json!({ "name": "Wohnen" })),
    )
    .await;
    let wohnen = v["tag"]["id"].as_str().unwrap().to_string();
    let a = upload_text(&c, &org, "miete.txt", "Miete").await;
    upload_text(&c, &org, "sonst.txt", "Sonst").await;
    call(
        &c,
        "POST",
        &format!("{base}/documents/{a}/tags"),
        Some(json!({ "tagId": wohnen })),
    )
    .await;

    let (s, v) = call(
        &c,
        "POST",
        &format!("{base}/folders"),
        Some(json!({ "name": "Haus", "tagIds": [wohnen] })),
    )
    .await;
    assert_eq!(s, Status::Created);
    let folder = v["folder"]["id"].as_str().unwrap().to_string();
    assert_eq!(v["folder"]["count"], 1);
    let (_, v) = call(
        &c,
        "GET",
        &format!("{base}/documents?folderId={folder}"),
        None,
    )
    .await;
    assert_eq!(v["documentsCount"], 1);
    assert_eq!(v["documents"][0]["name"], "miete.txt");
    let (s, _) = call(
        &c,
        "GET",
        &format!("{base}/documents?folderId=gibts-nicht"),
        None,
    )
    .await;
    assert_eq!(s, Status::NotFound);

    let (_, v) = call(&c, "GET", &format!("{base}/overview"), None).await;
    assert_eq!(v["role"], "owner");
    assert_eq!(v["folders"][0]["name"], "Haus");
    assert_eq!(v["tags"][0]["documentsCount"], 1);
    assert_eq!(v["reviewCount"], 0);
    let (_, v) = call(&c, "GET", &format!("{base}/members"), None).await;
    assert_eq!(v["members"][0]["username"], "alice");

    // Vorschaubild für Text: Platzhalter
    let res = c
        .get(format!("{base}/documents/{a}/thumbnail"))
        .dispatch()
        .await;
    assert_eq!(res.content_type(), Some(ContentType::SVG));
    drop(res);

    // Papierkorb: nur aus dem Papierkorb endgültig löschen; leeren
    let (s, _) = call(
        &c,
        "DELETE",
        &format!("{base}/documents/{a}/permanent"),
        None,
    )
    .await;
    assert_eq!(s, Status::BadRequest, "erst in den Papierkorb");
    call(&c, "DELETE", &format!("{base}/documents/{a}"), None).await;
    let (s, _) = call(
        &c,
        "DELETE",
        &format!("{base}/documents/{a}/permanent"),
        None,
    )
    .await;
    assert_eq!(s, Status::NoContent);
    let (_, v) = call(&c, "GET", &format!("{base}/documents"), None).await;
    let b = v["documents"][0]["id"].as_str().unwrap().to_string();
    call(&c, "DELETE", &format!("{base}/documents/{b}"), None).await;
    let (_, v) = call(&c, "POST", &format!("{base}/trash/empty"), None).await;
    assert_eq!(v["deleted"], 1);
    let (s, _) = call(&c, "DELETE", &format!("{base}/folders/{folder}"), None).await;
    assert_eq!(s, Status::NoContent);
    c.terminate().await;
}

//! Server-Tests mit Rockets lokalem Client: Ersteinrichtung, Anmeldung, Sitzungen,
//! Organisationen, CSRF-Schutz, Sicherheits-Header, Fehler, die Oberfläche (eingebettete
//! Dioxus-App) und die API mit Keys.

mod common;

use common::{call, login, logout, pool, register};
use nrzpaper::auth::api_key;
use nrzpaper::db::models::Permission;
use nrzpaper::services::users;
use rocket::http::{Accept, ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use serde_json::json;

async fn client() -> (tempfile::TempDir, Client) {
    common::empty().await
}

/// Erstes Konto anlegen und eine Organisation erstellen; gibt deren ID zurück.
async fn onboard(client: &Client) -> String {
    assert_eq!(register(client, "alice", "geheim-123").await, Status::Created);
    common::new_org(client, "Privat").await
}

#[rocket::async_test]
async fn first_start_leads_to_registration_and_first_user_is_admin() {
    let (_d, client) = client().await;
    {
        let (_, v) = call(&client, "GET", "/api/v1/auth/status", None).await;
        assert_eq!(v["needsSetup"], true);
        assert_eq!(v["loggedIn"], false);

        let res = client
            .post("/api/v1/auth/register")
            .header(ContentType::JSON)
            .body(json!({ "username": "alice", "password": "geheim-123" }).to_string())
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Created);
        let cookie = res.headers().get_one("Set-Cookie").unwrap().to_string();
        assert!(
            cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"),
            "Cookie-Flags fehlen: {cookie}"
        );
        assert!(!cookie.contains("geheim"), "Cookie enthält Klartext");
        let v: serde_json::Value = res.into_json().await.unwrap();
        assert_eq!(v["user"]["isAdmin"], true);

        let (_, v) = call(&client, "GET", "/api/v1/auth/status", None).await;
        assert_eq!((v["needsSetup"].clone(), v["loggedIn"].clone()), (json!(false), json!(true)));
        let user = users::by_username(pool(&client), "ALICE")
            .await
            .unwrap()
            .unwrap();
        assert!(user.is_admin);
        assert!(user.password_hash.starts_with("$argon2id$"));
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn registration_is_closed_after_the_first_user() {
    let (_d, client) = client().await;
    {
        onboard(&client).await;
        logout(&client).await;
        let (_, v) = call(&client, "GET", "/api/v1/auth/status", None).await;
        assert_eq!(v["registrationOpen"], false);
        assert_eq!(register(&client, "zweiter", "geheim-123").await, Status::Forbidden);

        // Ein Administrator kann sie öffnen
        login(&client, "alice", "geheim-123").await;
        let (s, _) = call(&client, "PUT", "/api/v1/settings/instance", Some(json!({ "registrationOpen": true }))).await;
        assert_eq!(s, Status::Ok);
        logout(&client).await;
        assert_eq!(register(&client, "zweiter", "geheim-123").await, Status::Created);
        let (_, v) = call(&client, "GET", "/api/v1/me", None).await;
        assert_eq!(v["user"]["isAdmin"], false);
        // … und Nicht-Admins kommen nicht an die Einstellungen
        let (s, _) = call(&client, "GET", "/api/v1/settings/instance", None).await;
        assert_eq!(s, Status::Forbidden);
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn invalid_registration_is_rejected() {
    let (_d, client) = client().await;
    {
        let (s, v) = call(
            &client,
            "POST",
            "/api/v1/auth/register",
            Some(json!({ "username": "ab", "password": "kurz" })),
        )
        .await;
        assert_eq!(s, Status::BadRequest);
        assert!(v["error"]["message"].as_str().unwrap().contains("Benutzername"));
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn login_logout_and_protected_data() {
    let (_d, client) = client().await;
    {
        let org = onboard(&client).await;
        let url = format!("/api/v1/organizations/{org}/overview");
        assert_eq!(call(&client, "GET", &url, None).await.0, Status::Ok);

        logout(&client).await;
        let (s, v) = call(&client, "GET", &url, None).await;
        assert_eq!(s, Status::Unauthorized);
        assert_eq!(v["error"]["code"], "unauthorized");
        // Seiten der Oberfläche gibt es trotzdem – die App leitet selbst zur Anmeldung
        assert_eq!(client.get(format!("/organizations/{org}")).dispatch().await.status(), Status::Ok);

        let start = std::time::Instant::now();
        let (s, _) = call(
            &client,
            "POST",
            "/api/v1/auth/login",
            Some(json!({ "username": "alice", "password": "falsch-falsch" })),
        )
        .await;
        assert_eq!(s, Status::Unauthorized);
        assert!(start.elapsed().as_millis() >= 300, "Fehlversuche werden gebremst");

        assert_eq!(login(&client, "ALICE", "geheim-123").await, Status::Ok);
        assert_eq!(call(&client, "GET", &url, None).await.0, Status::Ok);
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn foreign_organizations_are_invisible() {
    let (_d, client) = client().await;
    {
        let org = onboard(&client).await;
        users::create(pool(&client), "fremd", "geheim-456", "")
            .await
            .unwrap();
        logout(&client).await;
        login(&client, "fremd", "geheim-456").await;
        for url in [
            format!("/api/v1/organizations/{org}"),
            format!("/api/v1/organizations/{org}/overview"),
            format!("/api/v1/organizations/{org}/documents"),
        ] {
            assert_eq!(call(&client, "GET", &url, None).await.0, Status::NotFound, "{url}");
        }
        // Ohne eigene Organisation: leere Liste (die App bietet das Anlegen an)
        let (_, v) = call(&client, "GET", "/api/v1/me", None).await;
        assert_eq!(v["organizations"], json!([]));
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn cross_origin_changes_are_rejected() {
    let (_d, client) = client().await;
    {
        onboard(&client).await;
        let post = |origin: &'static str, name: &'static str| {
            client
                .post("/api/v1/organizations")
                .header(ContentType::JSON)
                .header(Accept::JSON)
                .header(Header::new("Host", "paper.local"))
                .header(Header::new("Origin", origin))
                .body(json!({ "name": name }).to_string())
                .dispatch()
        };
        assert_eq!(post("https://evil.example", "Boese").await.status(), Status::Forbidden);
        // Gleiche Herkunft ist erlaubt
        assert_eq!(post("http://paper.local", "Gewerbe").await.status(), Status::Created);
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn security_headers_errors_and_the_app() {
    let (_d, client) = client().await;
    {
        // Jede Seite der App bekommt index.html (oder den Hinweis, wenn nicht eingebaut)
        let res = client.get("/organizations/x/settings/ai").dispatch().await;
        assert!(
            res.status() == Status::Ok || res.status() == Status::ServiceUnavailable,
            "{}",
            res.status()
        );
        for h in [
            "Content-Security-Policy",
            "X-Content-Type-Options",
            "X-Frame-Options",
            "Referrer-Policy",
        ] {
            assert!(res.headers().contains(h), "{h} fehlt");
        }
        assert_eq!(res.content_type(), Some(ContentType::HTML));
        let csp = res.headers().get_one("Content-Security-Policy").unwrap().to_string();
        let script = csp.split(';').find(|d| d.trim().starts_with("script-src")).unwrap();
        assert!(script.contains("'wasm-unsafe-eval'") && !script.contains("unsafe-inline"), "{csp}");
        if res.status() == Status::Ok {
            assert_eq!(res.headers().get_one("Cache-Control"), Some("no-cache"));
            let html = res.into_string().await.unwrap();
            assert!(html.contains("<div id=\"main\">") && !html.contains("<script>"), "keine Inline-Skripte");
            // Dateien der App: lange zwischenspeichern, komprimiert, fehlende → 404
            let js = html.split("src=\"/./").nth(1).unwrap().split('"').next().unwrap().to_string();
            let res = client.get(format!("/{js}")).header(Header::new("Accept-Encoding", "gzip")).dispatch().await;
            assert_eq!(res.status(), Status::Ok);
            assert_eq!(res.headers().get_one("Content-Encoding"), Some("gzip"));
            assert!(res.headers().get_one("Cache-Control").unwrap().contains("immutable"));
        }
        assert_eq!(client.get("/assets/alt-123.js").dispatch().await.status(), Status::NotFound);
        assert_eq!(client.get("/static/gibt-es-nicht.js").dispatch().await.status(), Status::NotFound);

        let res = client.get("/api/v1/gibt-es-nicht").dispatch().await;
        assert_eq!(res.status(), Status::NotFound);
        assert_eq!(res.content_type(), Some(ContentType::JSON));
        assert!(res.into_string().await.unwrap().contains("\"not_found\""));
        // Fehler außerhalb der API als kleine Seite
        let res = client.get("/share/x/file").dispatch().await;
        assert_eq!(res.status(), Status::NotFound);
        assert_eq!(res.content_type(), Some(ContentType::HTML));
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn api_requires_valid_key_with_permissions() {
    let (_d, client) = client().await;
    {
        onboard(&client).await;
        logout(&client).await;
        let user = users::by_username(pool(&client), "alice")
            .await
            .unwrap()
            .unwrap();

        // Ohne Anmeldung
        let res = client.get("/api/v1/organizations").dispatch().await;
        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(res.content_type(), Some(ContentType::JSON));
        // Falscher Key
        let res = client
            .get("/api/v1/organizations")
            .header(Header::new("Authorization", "Bearer np_falsch"))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Unauthorized);

        // Key ohne passende Berechtigung
        let (_, read_docs) = api_key::create(
            pool(&client),
            &user.id,
            "nur Dokumente",
            &[Permission::DocumentsRead],
            None,
        )
        .await
        .unwrap();
        let res = client
            .get("/api/v1/organizations")
            .header(Header::new("Authorization", format!("Bearer {read_docs}")))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Forbidden);

        // Key mit Berechtigungen
        let (record, key) = api_key::create(
            pool(&client),
            &user.id,
            "cli",
            &[
                Permission::OrganizationsRead,
                Permission::OrganizationsCreate,
            ],
            None,
        )
        .await
        .unwrap();
        assert!(key.starts_with(&record.prefix) && record.key_hash != key);
        let auth = Header::new("Authorization", format!("Bearer {key}"));
        let res = client
            .get("/api/v1/organizations")
            .header(auth.clone())
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Ok);
        assert!(res.into_string().await.unwrap().contains("\"Privat\""));

        let res = client
            .post("/api/v1/organizations")
            .header(auth.clone())
            .header(ContentType::JSON)
            .body(r#"{"name":"Gewerbe"}"#)
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Created);
        let res = client
            .get("/api/v1/api-keys/current")
            .header(auth)
            .dispatch()
            .await;
        assert!(res
            .into_string()
            .await
            .unwrap()
            .contains("organizations:create"));

        // Abgelaufener Key
        let (_, old) = api_key::create(
            pool(&client),
            &user.id,
            "alt",
            &[Permission::OrganizationsRead],
            Some(1),
        )
        .await
        .unwrap();
        let res = client
            .get("/api/v1/organizations")
            .header(Header::new("Authorization", format!("Bearer {old}")))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Unauthorized);
    }
    client.terminate().await;
}

#[rocket::async_test]
async fn sessions_survive_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let cookie = {
        let client = Client::tracked(nrzpaper::build_with(common::figment(&dir)).await.unwrap())
            .await
            .unwrap();
        onboard(&client).await;
        let cookie = client
            .cookies()
            .get("nrzpaper_session")
            .unwrap()
            .clone()
            .into_owned();
        client.terminate().await;
        cookie
    };
    // Neuer Start: derselbe Cookie-Schlüssel aus der verschlüsselten Datenbank
    let client = Client::tracked(nrzpaper::build_with(common::figment(&dir)).await.unwrap())
        .await
        .unwrap();
    {
        let res = client.get("/api/v1/me").cookie(cookie).dispatch().await;
        assert_eq!(res.status(), Status::Ok, "Sitzung nach Neustart verloren");
    }
    client.terminate().await;
}

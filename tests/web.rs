//! Server-Tests mit Rockets lokalem Client: Ersteinrichtung, Anmeldung, Sitzungen,
//! Organisationen, CSRF-Schutz, Sicherheits-Header, Fehlerseiten und die API mit Keys.

use nrzpaper::auth::api_key;
use nrzpaper::db::models::Permission;
use nrzpaper::services::users;
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::{Client, LocalResponse};
use sqlx::SqlitePool;

async fn client() -> (tempfile::TempDir, Client) {
    let dir = tempfile::tempdir().unwrap();
    let figment = nrzpaper::config::figment()
        .merge(("data_dir", dir.path()))
        .merge(("db_key", "test"))
        .merge(("log_level", "off"));
    let client = Client::tracked(nrzpaper::build_with(figment).await.unwrap()).await.unwrap();
    (dir, client)
}

fn pool(client: &Client) -> &SqlitePool {
    client.rocket().state::<SqlitePool>().unwrap()
}

fn location<'a>(res: &'a LocalResponse<'_>) -> &'a str {
    res.headers().get_one("Location").unwrap_or("")
}

async fn post_form<'c>(client: &'c Client, uri: &'c str, body: &str) -> LocalResponse<'c> {
    client.post(uri).header(ContentType::Form).body(body.to_string()).dispatch().await
}

/// Erstes Konto anlegen und eine Organisation erstellen; gibt deren ID zurück.
async fn onboard(client: &Client) -> String {
    let res = post_form(client, "/register", "username=nico&password=geheim-123").await;
    assert_eq!(res.status(), Status::SeeOther);
    let res = post_form(client, "/organizations/new", "name=Privat").await;
    assert_eq!(res.status(), Status::SeeOther);
    location(&res).trim_start_matches("/organizations/").to_string()
}

#[rocket::async_test]
async fn first_start_leads_to_registration_and_first_user_is_admin() {
    let (_d, client) = client().await;
    let res = client.get("/").dispatch().await;
    assert_eq!(location(&res), "/register");
    assert_eq!(location(&client.get("/login").dispatch().await), "/register");

    let res = post_form(&client, "/register", "username=nico&password=geheim-123").await;
    assert_eq!(res.status(), Status::SeeOther);
    assert_eq!(location(&res), "/organizations/new");
    let cookie = res.headers().get_one("Set-Cookie").unwrap();
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"), "Cookie-Flags fehlen: {cookie}");
    assert!(!cookie.contains("geheim"), "Cookie enthält Klartext");

    let user = users::by_username(pool(&client), "NICO").await.unwrap().unwrap();
    assert!(user.is_admin);
    assert!(user.password_hash.starts_with("$argon2id$"));
}

#[rocket::async_test]
async fn registration_is_closed_after_the_first_user() {
    let (_d, client) = client().await;
    onboard(&client).await;
    post_form(&client, "/logout", "").await;
    assert_eq!(client.get("/register").dispatch().await.status(), Status::Forbidden);
    assert_eq!(post_form(&client, "/register", "username=zweiter&password=geheim-123").await.status(), Status::Forbidden);
}

#[rocket::async_test]
async fn invalid_registration_shows_the_form_again() {
    let (_d, client) = client().await;
    let res = post_form(&client, "/register", "username=ab&password=kurz").await;
    assert_eq!(res.status(), Status::BadRequest);
    assert!(res.into_string().await.unwrap().contains("Benutzername"));
}

#[rocket::async_test]
async fn login_logout_and_protected_pages() {
    let (_d, client) = client().await;
    let org = onboard(&client).await;
    let page = format!("/organizations/{org}");

    let res = client.get(page.as_str()).dispatch().await;
    assert_eq!(res.status(), Status::Ok);
    assert!(res.into_string().await.unwrap().contains("Privat"));

    post_form(&client, "/logout", "").await;
    let res = client.get(page.as_str()).dispatch().await;
    assert_eq!(res.status(), Status::SeeOther);
    assert_eq!(location(&res), format!("/login?next=/organizations/{org}"));

    let res = post_form(&client, "/login", "username=nico&password=falsch-falsch").await;
    assert_eq!(res.status(), Status::Unauthorized);
    assert!(res.into_string().await.unwrap().contains("stimmt nicht"));

    let res = post_form(&client, "/login", &format!("username=nico&password=geheim-123&next=/organizations/{org}")).await;
    assert_eq!(location(&res), page);
    assert_eq!(client.get(page.as_str()).dispatch().await.status(), Status::Ok);

    // Kein offener Redirect nach der Anmeldung
    post_form(&client, "/logout", "").await;
    let res = post_form(&client, "/login", "username=nico&password=geheim-123&next=//evil.example").await;
    assert_eq!(location(&res), "/");
}

#[rocket::async_test]
async fn foreign_organizations_are_invisible() {
    let (_d, client) = client().await;
    let org = onboard(&client).await;
    users::create(pool(&client), "fremd", "geheim-456", "").await.unwrap();
    post_form(&client, "/logout", "").await;
    post_form(&client, "/login", "username=fremd&password=geheim-456").await;
    assert_eq!(client.get(format!("/organizations/{org}")).dispatch().await.status(), Status::NotFound);
    // Ohne eigene Organisation geht es zum Anlegen
    assert_eq!(location(&client.get("/").dispatch().await), "/organizations/new");
}

#[rocket::async_test]
async fn cross_origin_changes_are_rejected() {
    let (_d, client) = client().await;
    onboard(&client).await;
    let res = client
        .post("/organizations/new")
        .header(ContentType::Form)
        .header(Header::new("Host", "paper.local"))
        .header(Header::new("Origin", "https://evil.example"))
        .body("name=Boese")
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::Forbidden);
    // Gleiche Herkunft ist erlaubt
    let res = client
        .post("/organizations/new")
        .header(ContentType::Form)
        .header(Header::new("Host", "paper.local"))
        .header(Header::new("Origin", "http://paper.local"))
        .body("name=Gewerbe")
        .dispatch()
        .await;
    assert_eq!(res.status(), Status::SeeOther);
}

#[rocket::async_test]
async fn security_headers_and_error_pages() {
    let (_d, client) = client().await;
    let res = client.get("/gibt-es-nicht").dispatch().await;
    assert_eq!(res.status(), Status::NotFound);
    for h in ["Content-Security-Policy", "X-Content-Type-Options", "X-Frame-Options", "Referrer-Policy"] {
        assert!(res.headers().contains(h), "{h} fehlt");
    }
    assert_eq!(res.content_type(), Some(ContentType::HTML));

    let res = client.get("/api/v1/gibt-es-nicht").dispatch().await;
    assert_eq!(res.status(), Status::NotFound);
    assert_eq!(res.content_type(), Some(ContentType::JSON));
    assert!(res.into_string().await.unwrap().contains("\"not_found\""));
}

#[rocket::async_test]
async fn api_requires_valid_key_with_permissions() {
    let (_d, client) = client().await;
    onboard(&client).await;
    post_form(&client, "/logout", "").await;
    let user = users::by_username(pool(&client), "nico").await.unwrap().unwrap();

    // Ohne Anmeldung
    let res = client.get("/api/v1/organizations").dispatch().await;
    assert_eq!(res.status(), Status::Unauthorized);
    assert_eq!(res.content_type(), Some(ContentType::JSON));
    // Falscher Key
    let res = client.get("/api/v1/organizations").header(Header::new("Authorization", "Bearer np_falsch")).dispatch().await;
    assert_eq!(res.status(), Status::Unauthorized);

    // Key ohne passende Berechtigung
    let (_, read_docs) = api_key::create(pool(&client), &user.id, "nur Dokumente", &[Permission::DocumentsRead], None).await.unwrap();
    let res = client.get("/api/v1/organizations").header(Header::new("Authorization", format!("Bearer {read_docs}"))).dispatch().await;
    assert_eq!(res.status(), Status::Forbidden);

    // Key mit Berechtigungen
    let (record, key) = api_key::create(pool(&client), &user.id, "cli", &[Permission::OrganizationsRead, Permission::OrganizationsCreate], None)
        .await
        .unwrap();
    assert!(key.starts_with(&record.prefix) && record.key_hash != key);
    let auth = Header::new("Authorization", format!("Bearer {key}"));
    let res = client.get("/api/v1/organizations").header(auth.clone()).dispatch().await;
    assert_eq!(res.status(), Status::Ok);
    assert!(res.into_string().await.unwrap().contains("\"Privat\""));

    let res = client.post("/api/v1/organizations").header(auth.clone()).header(ContentType::JSON).body(r#"{"name":"Gewerbe"}"#).dispatch().await;
    assert_eq!(res.status(), Status::Created);
    let res = client.get("/api/v1/api-keys/current").header(auth).dispatch().await;
    assert!(res.into_string().await.unwrap().contains("organizations:create"));

    // Abgelaufener Key
    let (_, old) = api_key::create(pool(&client), &user.id, "alt", &[Permission::OrganizationsRead], Some(1)).await.unwrap();
    let res = client.get("/api/v1/organizations").header(Header::new("Authorization", format!("Bearer {old}"))).dispatch().await;
    assert_eq!(res.status(), Status::Unauthorized);
}

#[rocket::async_test]
async fn sessions_survive_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let figment = || {
        nrzpaper::config::figment().merge(("data_dir", dir.path())).merge(("db_key", "test")).merge(("log_level", "off"))
    };
    let cookie = {
        let client = Client::tracked(nrzpaper::build_with(figment()).await.unwrap()).await.unwrap();
        onboard(&client).await;
        client.cookies().get("nrzpaper_session").unwrap().clone().into_owned()
    };
    // Neuer Start: derselbe Cookie-Schlüssel aus der verschlüsselten Datenbank
    let client = Client::tracked(nrzpaper::build_with(figment()).await.unwrap()).await.unwrap();
    let res = client.get("/").cookie(cookie).dispatch().await;
    assert!(location(&res).starts_with("/organizations/"), "Sitzung nach Neustart verloren: {}", location(&res));
}

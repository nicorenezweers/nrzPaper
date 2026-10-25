//! JSON-Endpunkte für die Dioxus-Oberfläche (`ui/`): Anmeldung, eigenes Konto, Einstellungen
//! (Instanz, KI/OpenRouter), Ordner, Tagging-Regeln, Eigenschaften, Mitglieder, Papierkorb und
//! Vorschaubilder. Die Oberfläche spricht mit Sitzungs-Cookie; ändernde Anfragen von fremden
//! Seiten weist der Guard ab (Origin-Prüfung).

use crate::ai::{self, AiConfig, AiSettings};
use crate::auth::guards::{AdminUser, CurrentUser, MaybeUser, UserAgent};
use crate::auth::{api_key, session};
use crate::config::Config;
use crate::db::models::{now, Permission, Role};
use crate::error::{AppError, AppResult};
use crate::services::{
    documents, folders, inbox, mail, organizations, properties, scan, settings, tagging_rules,
    tags, users,
};
use rocket::http::{ContentType, CookieJar, Header, Status};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Responder, Route, State};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::time::Duration;

pub fn routes() -> Vec<Route> {
    routes![
        auth_status,
        login,
        register,
        logout,
        me,
        change_password,
        list_keys,
        create_key,
        delete_key,
        get_scanner,
        put_scanner,
        list_mail,
        create_mail,
        update_mail,
        delete_mail,
        fetch_mail,
        test_mail,
        get_ai,
        put_ai,
        ai_models,
        get_instance,
        put_instance,
        list_users,
        overview,
        list_folders,
        create_folder,
        update_folder,
        delete_folder,
        list_rules,
        create_rule,
        update_rule,
        delete_rule,
        apply_rule,
        list_properties,
        create_property,
        update_property,
        delete_property,
        order_properties,
        document_properties,
        set_document_property,
        thumbnail,
        delete_permanently,
        empty_trash,
        shared
    ]
}

async fn member(pool: &SqlitePool, user: &CurrentUser, org: &str) -> AppResult<Role> {
    Ok(organizations::membership(pool, &user.0.id, org).await?.1)
}

fn user_json(u: &crate::db::models::User) -> Value {
    json!({ "id": u.id, "username": u.username, "displayName": u.display_name, "isAdmin": u.is_admin })
}

// ------------------------------------------------------------------ Anmeldung

#[get("/auth/status")]
async fn auth_status(
    user: MaybeUser,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    Ok(json!({
        "loggedIn": user.0.is_some(),
        "needsSetup": users::count(pool).await? == 0,
        "registrationOpen": users::registration_allowed(pool, config).await?,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

#[post("/auth/login", data = "<body>")]
async fn login(
    body: Json<Credentials>,
    cookies: &CookieJar<'_>,
    ua: UserAgent,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    match users::authenticate(pool, &body.username, &body.password).await? {
        Some(user) => {
            let token = session::create(pool, &user.id, &ua.0).await?;
            session::set_cookie(cookies, token, config.cookie_secure);
            tracing::info!(user = %user.username, "Anmeldung");
            Ok(json!({ "user": user_json(&user) }))
        }
        None => {
            // Bremst Durchprobieren von Passwörtern.
            tokio::time::sleep(Duration::from_millis(400)).await;
            tracing::warn!(username = %body.username, "Anmeldung fehlgeschlagen");
            Err(AppError::Unauthorized)
        }
    }
}

#[post("/auth/register", data = "<body>")]
async fn register(
    body: Json<Credentials>,
    cookies: &CookieJar<'_>,
    ua: UserAgent,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<(Status, Value)> {
    if !users::registration_allowed(pool, config).await? {
        return Err(AppError::forbidden(
            "Die Registrierung ist geschlossen – ein Administrator muss dein Konto anlegen.",
        ));
    }
    let user = users::create(pool, &body.username, &body.password, "").await?;
    let token = session::create(pool, &user.id, &ua.0).await?;
    session::set_cookie(cookies, token, config.cookie_secure);
    tracing::info!(user = %user.username, admin = user.is_admin, "Konto angelegt");
    Ok((Status::Created, json!({ "user": user_json(&user) })))
}

#[post("/auth/logout")]
async fn logout(cookies: &CookieJar<'_>, pool: &State<SqlitePool>) -> AppResult<Value> {
    if let Some(token) = session::token_from(cookies) {
        session::destroy(pool, &token).await?;
    }
    session::clear_cookie(cookies);
    Ok(json!({ "ok": true }))
}

// ------------------------------------------------------------------ Eigenes Konto

#[get("/me")]
async fn me(user: CurrentUser, pool: &State<SqlitePool>) -> AppResult<Value> {
    let orgs: Vec<Value> = organizations::list_for_user(pool, &user.0.id)
        .await?
        .into_iter()
        .map(|(o, role)| json!({ "id": o.id, "name": o.name, "role": role }))
        .collect();
    Ok(json!({ "user": user_json(&user.0), "organizations": orgs }))
}

#[derive(Deserialize)]
struct PasswordChange {
    current: String,
    new: String,
    confirm: String,
}

/// Passwort ändern; alle anderen Sitzungen enden, diese bekommt eine neue.
#[put("/me/password", data = "<body>")]
async fn change_password(
    user: CurrentUser,
    body: Json<PasswordChange>,
    cookies: &CookieJar<'_>,
    ua: UserAgent,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    users::change_password(pool, &user.0, &body.current, &body.new, &body.confirm).await?;
    let token = session::create(pool, &user.0.id, &ua.0).await?;
    session::set_cookie(cookies, token, config.cookie_secure);
    Ok(json!({ "ok": true }))
}

#[get("/me/api-keys")]
async fn list_keys(user: CurrentUser, pool: &State<SqlitePool>) -> AppResult<Value> {
    Ok(json!({ "apiKeys": api_key::list(pool, &user.0.id).await?, "permissions": Permission::ALL }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewKey {
    name: String,
    permissions: Vec<Permission>,
    expires_in_days: Option<u32>,
}

/// Neuer API-Key – der Key steht nur in dieser Antwort.
#[post("/me/api-keys", data = "<body>")]
async fn create_key(
    user: CurrentUser,
    body: Json<NewKey>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    let expires = body
        .expires_in_days
        .filter(|d| (1..=3650).contains(d))
        .map(|d| now() + d as i64 * 86_400);
    let (record, key) =
        api_key::create(pool, &user.0.id, &body.name, &body.permissions, expires).await?;
    Ok((Status::Created, json!({ "apiKey": record, "key": key })))
}

#[delete("/me/api-keys/<id>")]
async fn delete_key(user: CurrentUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    api_key::delete(pool, &user.0.id, id).await?;
    Ok(Status::NoContent)
}

#[get("/me/scanner")]
async fn get_scanner(
    user: CurrentUser,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    Ok(
        json!({ "scanner": scan::settings(pool, config, &user.0.id).await?, "subnets": config.scan_subnets }),
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScannerInput {
    scanner_url: String,
    color: bool,
    dpi: i64,
}

#[put("/me/scanner", data = "<body>")]
async fn put_scanner(
    user: CurrentUser,
    body: Json<ScannerInput>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    scan::save_settings(pool, &user.0.id, &body.scanner_url, body.color, body.dpi).await?;
    Ok(json!({ "scanner": scan::settings(pool, config, &user.0.id).await? }))
}

// ------------------------------------------------------------------ E-Mail-Eingang

#[get("/me/mail-accounts")]
async fn list_mail(user: CurrentUser, pool: &State<SqlitePool>) -> AppResult<Value> {
    Ok(json!({ "mailAccounts": mail::list(pool, &user.0.id).await? }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MailInput {
    #[serde(default)]
    name: String,
    organization_id: String,
    host: String,
    port: i64,
    security: String,
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    folder: String,
    #[serde(default)]
    mark_seen: bool,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    enabled: bool,
    interval_minutes: i64,
}

impl MailInput {
    fn input(&self) -> mail::AccountInput {
        mail::AccountInput {
            name: self.name.clone(),
            organization_id: self.organization_id.clone(),
            host: self.host.clone(),
            port: self.port,
            security: self.security.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            folder: self.folder.clone(),
            mark_seen: self.mark_seen,
            tags: self.tags.clone(),
            enabled: self.enabled,
            interval_minutes: self.interval_minutes,
        }
    }
}

#[post("/me/mail-accounts", data = "<body>")]
async fn create_mail(
    user: CurrentUser,
    body: Json<MailInput>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    organizations::membership(pool, &user.0.id, &body.organization_id).await?;
    Ok((
        Status::Created,
        json!({ "mailAccount": mail::create(pool, &user.0.id, &body.input()).await? }),
    ))
}

#[put("/me/mail-accounts/<id>", data = "<body>")]
async fn update_mail(
    user: CurrentUser,
    id: &str,
    body: Json<MailInput>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    organizations::membership(pool, &user.0.id, &body.organization_id).await?;
    Ok(json!({ "mailAccount": mail::update(pool, &user.0.id, id, &body.input()).await? }))
}

#[delete("/me/mail-accounts/<id>")]
async fn delete_mail(user: CurrentUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Status> {
    mail::delete(pool, &user.0.id, id).await?;
    Ok(Status::NoContent)
}

#[post("/me/mail-accounts/<id>/fetch")]
async fn fetch_mail(
    user: CurrentUser,
    id: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let account = mail::get(pool, &user.0.id, id).await?;
    let ai = AiConfig::load(pool, config).await;
    let report = mail::run(pool, &ai, &account)
        .await
        .map_err(AppError::bad)?;
    Ok(json!({ "report": report }))
}

#[post("/me/mail-accounts/<id>/test")]
async fn test_mail(
    user: CurrentUser,
    id: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let account = mail::get(pool, &user.0.id, id).await?;
    let ai = AiConfig::load(pool, config).await;
    mail::fetch(pool, &ai, &account, true)
        .await
        .map_err(AppError::bad)?;
    Ok(json!({ "ok": true }))
}

// ------------------------------------------------------------------ Einstellungen (Administratoren)

/// KI-Einstellungen für die Oberfläche – Schlüssel selbst verlassen den Server nie, nur ob
/// einer hinterlegt ist und Anfang/Ende zur Wiedererkennung.
async fn ai_json(pool: &SqlitePool, config: &Config) -> AppResult<Value> {
    let stored = settings::get::<AiSettings>(pool, ai::SETTINGS_KEY)
        .await?
        .unwrap_or_default();
    let effective = AiConfig::with_settings(config, Some(&stored));
    let providers: Vec<Value> = ai::PROVIDERS
        .iter()
        .map(|p| {
            let key = stored.key_for(p.id);
            let mut v = json!(p);
            v["hasKey"] = json!(!key.is_empty());
            v["keyHint"] = json!(ai::hint(key));
            v["customUrl"] = json!(stored.urls.get(p.id).cloned().unwrap_or_default());
            v
        })
        .collect();
    Ok(json!({
        "disabled": stored.disabled,
        "provider": effective.provider.id,
        "model": stored.model,
        "fallbacks": stored.fallbacks,
        "textOnly": stored.text_only,
        "language": if stored.language.is_empty() { "de".to_string() } else { stored.language.clone() },
        "effectiveModel": effective.model,
        "effectiveUrl": effective.url,
        "effectiveFallbacks": effective.fallbacks,
        "defaultModel": effective.provider.model,
        "defaultFallbacks": if effective.provider.id == "openrouter" { json!(ai::FALLBACK_MODELS) } else { json!([]) },
        "keySource": AiConfig::key_source(config, Some(&stored)),
        "keyHint": effective.key_hint(),
        "available": effective.available(),
        "missing": effective.missing(),
        "providers": providers,
    }))
}

#[get("/settings/ai")]
async fn get_ai(
    _admin: AdminUser,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    ai_json(pool, config).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiInput {
    disabled: bool,
    /// Anbieter (leer = bisheriger)
    #[serde(default)]
    provider: String,
    model: String,
    fallbacks: Vec<String>,
    /// Neuer Schlüssel für den Anbieter; leer lassen = bisherigen behalten
    #[serde(default)]
    key: String,
    /// Eigenen Schlüssel des Anbieters entfernen
    #[serde(default)]
    clear_key: bool,
    /// Eigene Adresse der API (leer = Vorgabe des Anbieters)
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    text_only: bool,
    /// Sprache der Vorschläge (Code oder "auto"); fehlt = bisherige
    #[serde(default)]
    language: Option<String>,
}

#[put("/settings/ai", data = "<body>")]
async fn put_ai(
    _admin: AdminUser,
    body: Json<AiInput>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let mut stored = settings::get::<AiSettings>(pool, ai::SETTINGS_KEY)
        .await?
        .unwrap_or_default();
    if !body.provider.trim().is_empty() {
        let p = ai::provider(body.provider.trim());
        if p.id != body.provider.trim() {
            return Err(AppError::bad("Diesen KI-Anbieter gibt es nicht."));
        }
        stored.provider = p.id.to_string();
    }
    let id = ai::provider(stored.provider_id()).id;
    let key = body.key.trim();
    if !key.is_empty() {
        if key.len() < 8 || key.len() > 4096 || key.contains(char::is_whitespace) {
            return Err(AppError::bad("Das sieht nicht wie ein API-Schlüssel aus."));
        }
        if id == "openrouter" && !key.starts_with("sk-") {
            return Err(AppError::bad(
                "Das sieht nicht wie ein OpenRouter-Schlüssel aus (sk-or-…).",
            ));
        }
        stored.keys.insert(id.to_string(), key.to_string());
    } else if body.clear_key {
        stored.keys.remove(id);
        if id == "openrouter" {
            stored.key.clear();
        }
    }
    if let Some(url) = body.url.as_deref() {
        let url = url.trim().trim_end_matches('/');
        if url.is_empty() {
            stored.urls.remove(id);
        } else if !(url.starts_with("http://") || url.starts_with("https://")) || url.len() > 500 {
            return Err(AppError::bad("Die Adresse muss mit http:// oder https:// beginnen."));
        } else {
            stored.urls.insert(id.to_string(), url.to_string());
        }
    }
    let clean = |m: &str| {
        m.trim()
            .chars()
            .filter(|c| !c.is_whitespace())
            .take(200)
            .collect::<String>()
    };
    stored.model = clean(&body.model);
    stored.fallbacks = body
        .fallbacks
        .iter()
        .map(|m| clean(m))
        .filter(|m| !m.is_empty())
        .take(10)
        .collect();
    stored.disabled = body.disabled;
    stored.text_only = body.text_only;
    if let Some(l) = body.language.as_deref() {
        if l != "auto" && !ai::SUGGESTION_LANGUAGES.iter().any(|(c, _)| *c == l) {
            return Err(AppError::bad("Diese Sprache gibt es nicht."));
        }
        stored.language = l.to_string();
    }
    settings::set(pool, ai::SETTINGS_KEY, &stored).await?;
    ai_json(pool, config).await
}

/// Modelle des gewählten Anbieters (oder eines anderen: `?provider=`) für die Auswahl.
#[get("/settings/ai/models?<provider>")]
async fn ai_models(
    _admin: AdminUser,
    provider: Option<&str>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let mut stored = settings::get::<AiSettings>(pool, ai::SETTINGS_KEY)
        .await?
        .unwrap_or_default();
    if let Some(p) = provider.filter(|p| !p.is_empty()) {
        stored.provider = ai::provider(p).id.to_string();
        stored.model.clear();
    }
    let ai = AiConfig::with_settings(config, Some(&stored));
    let models = ai::models(&ai).await.map_err(AppError::bad)?;
    Ok(json!({ "models": models }))
}

#[get("/settings/instance")]
async fn get_instance(
    _admin: AdminUser,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let open = settings::get::<bool>(pool, settings::REGISTRATION_OPEN)
        .await?
        .unwrap_or(config.registration_open);
    Ok(
        json!({ "registrationOpen": open, "trashRetentionDays": config.trash_retention_days, "ocrLanguages": config.ocr_languages, "version": env!("CARGO_PKG_VERSION") }),
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstanceInput {
    registration_open: bool,
}

#[put("/settings/instance", data = "<body>")]
async fn put_instance(
    admin: AdminUser,
    body: Json<InstanceInput>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    settings::set(pool, settings::REGISTRATION_OPEN, &body.registration_open).await?;
    get_instance(admin, pool, config).await
}

#[get("/settings/users")]
async fn list_users(_admin: AdminUser, pool: &State<SqlitePool>) -> AppResult<Value> {
    let rows: Vec<(String, String, bool, i64)> =
        sqlx::query_as("SELECT id, username, is_admin, created_at FROM users ORDER BY created_at")
            .fetch_all(pool.inner())
            .await?;
    Ok(
        json!({ "users": rows.into_iter().map(|(id, u, a, c)| json!({ "id": id, "username": u, "isAdmin": a, "createdAt": c })).collect::<Vec<_>>() }),
    )
}

// ------------------------------------------------------------------ Organisation: Überblick

/// Alles, was die Seitenleiste braucht: Zähler der Inbox, Ordnerbaum, Tags mit Anzahl.
#[get("/organizations/<org>/overview")]
async fn overview(
    user: CurrentUser,
    org: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Value> {
    let role = member(pool, &user, org).await?;
    let tag_list: Vec<Value> = tags::list(pool, org)
        .await?
        .into_iter()
        .map(|(t, n)| {
            let mut v = json!(t);
            v["documentsCount"] = json!(n);
            v
        })
        .collect();
    let ai = AiConfig::load(pool, config).await;
    Ok(json!({
        "role": role,
        "reviewCount": inbox::open_count(pool, org).await?,
        "remindersDue": crate::services::reminders::due_count(pool, &user.0.id, org).await?,
        "folders": folders::tree(pool, org).await?,
        "tags": tag_list,
        "aiAvailable": ai.available(),
    }))
}

// ------------------------------------------------------------------ Ordner

#[get("/organizations/<org>/folders")]
async fn list_folders(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    member(pool, &user, org).await?;
    Ok(json!({ "folders": folders::tree(pool, org).await? }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FolderInput {
    name: String,
    parent_id: Option<String>,
    #[serde(default)]
    tag_ids: Vec<String>,
}

#[post("/organizations/<org>/folders", data = "<body>")]
async fn create_folder(
    user: CurrentUser,
    org: &str,
    body: Json<FolderInput>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    member(pool, &user, org).await?;
    let parent = body.parent_id.as_deref().filter(|p| !p.is_empty());
    let id = folders::create(pool, org, &body.name, parent, &body.tag_ids).await?;
    Ok((
        Status::Created,
        json!({ "folder": folders::get(pool, org, &id).await? }),
    ))
}

#[put("/organizations/<org>/folders/<id>", data = "<body>")]
async fn update_folder(
    user: CurrentUser,
    org: &str,
    id: &str,
    body: Json<FolderInput>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    let parent = body.parent_id.as_deref().filter(|p| !p.is_empty());
    folders::update(pool, org, id, &body.name, parent, &body.tag_ids).await?;
    Ok(json!({ "folder": folders::get(pool, org, id).await? }))
}

#[delete("/organizations/<org>/folders/<id>")]
async fn delete_folder(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    member(pool, &user, org).await?;
    folders::delete(pool, org, id).await?;
    Ok(Status::NoContent)
}

// ------------------------------------------------------------------ Tagging-Regeln

#[get("/organizations/<org>/tagging-rules")]
async fn list_rules(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    member(pool, &user, org).await?;
    Ok(json!({ "taggingRules": tagging_rules::list(pool, org).await? }))
}

#[post("/organizations/<org>/tagging-rules", data = "<body>")]
async fn create_rule(
    user: CurrentUser,
    org: &str,
    body: Json<tagging_rules::RuleInput>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    member(pool, &user, org).await?;
    Ok((
        Status::Created,
        json!({ "taggingRule": tagging_rules::create(pool, org, &body).await? }),
    ))
}

#[put("/organizations/<org>/tagging-rules/<id>", data = "<body>")]
async fn update_rule(
    user: CurrentUser,
    org: &str,
    id: &str,
    body: Json<tagging_rules::RuleInput>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    Ok(json!({ "taggingRule": tagging_rules::update(pool, org, id, &body).await? }))
}

#[delete("/organizations/<org>/tagging-rules/<id>")]
async fn delete_rule(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    member(pool, &user, org).await?;
    tagging_rules::delete(pool, org, id).await?;
    Ok(Status::NoContent)
}

/// Regel rückwirkend auf alle Dokumente anwenden.
#[post("/organizations/<org>/tagging-rules/<id>/apply")]
async fn apply_rule(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    let (matched, added) = tagging_rules::apply_to_all(pool, org, id).await?;
    Ok(json!({ "matched": matched, "tagsAdded": added }))
}

// ------------------------------------------------------------------ Eigenschaften

#[get("/organizations/<org>/custom-properties")]
async fn list_properties(
    user: CurrentUser,
    org: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    Ok(json!({ "customProperties": properties::definitions(pool, org).await? }))
}

#[post("/organizations/<org>/custom-properties", data = "<body>")]
async fn create_property(
    user: CurrentUser,
    org: &str,
    body: Json<properties::DefinitionInput>,
    pool: &State<SqlitePool>,
) -> AppResult<(Status, Value)> {
    member(pool, &user, org).await?;
    Ok((
        Status::Created,
        json!({ "customProperty": properties::create_definition(pool, org, &body).await? }),
    ))
}

#[put(
    "/organizations/<org>/custom-properties/<id>",
    data = "<body>",
    rank = 2
)]
async fn update_property(
    user: CurrentUser,
    org: &str,
    id: &str,
    body: Json<properties::DefinitionInput>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    Ok(json!({ "customProperty": properties::update_definition(pool, org, id, &body).await? }))
}

#[delete("/organizations/<org>/custom-properties/<id>")]
async fn delete_property(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    member(pool, &user, org).await?;
    properties::delete_definition(pool, org, id).await?;
    Ok(Status::NoContent)
}

#[derive(Deserialize)]
struct Order {
    ids: Vec<String>,
}

#[put("/organizations/<org>/custom-properties/order", data = "<body>")]
async fn order_properties(
    user: CurrentUser,
    org: &str,
    body: Json<Order>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    properties::reorder(pool, org, &body.ids).await?;
    Ok(json!({ "customProperties": properties::definitions(pool, org).await? }))
}

#[get("/organizations/<org>/documents/<id>/custom-properties")]
async fn document_properties(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    documents::get(pool, org, id).await?;
    Ok(json!({ "values": properties::values(pool, org, id).await? }))
}

#[derive(Deserialize)]
struct Values {
    values: Vec<String>,
}

#[put(
    "/organizations/<org>/documents/<id>/custom-properties/<def>",
    data = "<body>"
)]
async fn set_document_property(
    user: CurrentUser,
    org: &str,
    id: &str,
    def: &str,
    body: Json<Values>,
    pool: &State<SqlitePool>,
) -> AppResult<Value> {
    member(pool, &user, org).await?;
    documents::get(pool, org, id).await?;
    properties::set_values(pool, org, id, def, &body.values).await?;
    Ok(json!({ "values": properties::values(pool, org, id).await? }))
}

// ------------------------------------------------------------------ Mitglieder, Papierkorb, Vorschau


/// Vorschaubild (JPEG) oder Platzhalter (SVG).
#[derive(Responder)]
enum Thumb {
    Image(Vec<u8>, ContentType, Header<'static>),
    Placeholder(String, ContentType, Header<'static>),
}

#[get("/organizations/<org>/documents/<id>/thumbnail")]
async fn thumbnail(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Thumb> {
    member(pool, &user, org).await?;
    let doc = documents::get(pool, org, id).await?;
    Ok(match crate::services::thumbnails::get(pool, &doc).await? {
        Some(jpeg) => Thumb::Image(
            jpeg,
            ContentType::JPEG,
            Header::new("Cache-Control", "private, max-age=604800"),
        ),
        None => Thumb::Placeholder(
            crate::services::thumbnails::placeholder(crate::services::thumbnails::type_label(&doc)),
            ContentType::SVG,
            Header::new("Cache-Control", "private, max-age=3600"),
        ),
    })
}

/// Endgültig löschen – nur aus dem Papierkorb.
#[delete("/organizations/<org>/documents/<id>/permanent")]
async fn delete_permanently(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Status> {
    member(pool, &user, org).await?;
    documents::delete_permanently(pool, org, id).await?;
    Ok(Status::NoContent)
}

/// Papierkorb leeren (Besitzer und Admins).
#[post("/organizations/<org>/trash/empty")]
async fn empty_trash(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    if !member(pool, &user, org).await?.can_manage() {
        return Err(AppError::forbidden(
            "Nur Besitzer und Admins können den Papierkorb leeren.",
        ));
    }
    let n =
        sqlx::query("DELETE FROM documents WHERE organization_id = ? AND deleted_at IS NOT NULL")
            .bind(org)
            .execute(pool.inner())
            .await?
            .rows_affected();
    Ok(json!({ "deleted": n }))
}

/// Öffentliche Freigabe: Angaben zum Dokument (ohne Anmeldung). Zählt einen Aufruf; die Datei
/// selbst liefert `/share/<token>/file`.
#[get("/share/<token>")]
async fn shared(token: &str, pool: &State<SqlitePool>) -> AppResult<Value> {
    let (link, doc) = crate::services::share_links::resolve(pool, token, true).await?;
    Ok(json!({
        "document": {
            "name": doc.name, "mimeType": doc.mime_type, "size": doc.size, "pageCount": doc.page_count,
            "content": if doc.mime_type.starts_with("text/") { doc.content } else { String::new() },
        },
        "expiresAt": link.expires_at,
        "fileUrl": format!("/share/{token}/file"),
    }))
}

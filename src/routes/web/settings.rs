//! Konto (Passwort, API-Keys) und Einstellungen einer Organisation (Name, Mitglieder,
//! Webhooks). Neue Geheimnisse (Key, Webhook-Secret) zeigt die Antwort auf das Formular
//! direkt an – einmal, ohne Umweg über URL oder Cookie.

use super::pages::shell;
use super::Page;
use crate::auth::guards::{CurrentUser, UserAgent};
use crate::auth::{api_key, session};
use crate::config::Config;
use crate::db::models::{now, Permission, Role, User, WebhookEvent};
use crate::error::{AppError, AppResult};
use crate::services::{organizations, users, webhooks};
use crate::templates::{render, AccountPage, HookRow, MemberRow, SettingsPage};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::status::Custom;
use rocket::response::Redirect;
use rocket::{get, post, routes, FromForm, Route, State};
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![
        account,
        change_password,
        create_key,
        delete_key,
        settings,
        rename_org,
        delete_org,
        create_webhook,
        toggle_webhook,
        delete_webhook,
        create_mail,
        update_mail,
        delete_mail,
        fetch_mail,
        test_mail
    ]
}

// ------------------------------------------------------------------ Konto

async fn account_page(
    pool: &SqlitePool,
    config: &Config,
    user: User,
    org: &str,
    new_key: Option<String>,
    notice: Option<String>,
    error: Option<&AppError>,
) -> AppResult<Page> {
    let keys = api_key::list(pool, &user.id).await?;
    let scan = crate::services::scan::settings(pool, config, &user.id).await?;
    let ai = crate::ai::AiConfig::load(pool, config).await;
    let mail_accounts = crate::services::mail::list(pool, &user.id).await?;
    let shell = shell(pool, user, org, "account", "").await?;
    let page = render(&AccountPage {
        shell,
        scan,
        ai_available: ai.available(),
        ai_model: ai.model,
        mail_accounts,
        keys,
        new_key,
        notice,
        error: error.map(AppError::public_message),
    })?;
    Ok(match error {
        Some(e) => Page::Status(Custom(e.status(), page)),
        None => Page::Html(page),
    })
}

#[get("/organizations/<org>/account")]
async fn account(
    user: CurrentUser,
    org: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    account_page(pool, config, user.0, org, None, None, None).await
}

#[derive(FromForm)]
struct PasswordForm<'r> {
    current: &'r str,
    new: &'r str,
    confirm: &'r str,
}

#[post("/organizations/<org>/account/password", data = "<form>")]
#[allow(clippy::too_many_arguments)]
async fn change_password(
    user: CurrentUser,
    org: &str,
    form: Form<PasswordForm<'_>>,
    ua: UserAgent,
    cookies: &CookieJar<'_>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    organizations::membership(pool, &user.0.id, org).await?;
    match users::change_password(pool, &user.0, form.current, form.new, form.confirm).await {
        Ok(()) => {
            // Alle Sitzungen sind weg – diese hier bekommt sofort eine neue.
            let token = session::create(pool, &user.0.id, &ua.0).await?;
            session::set_cookie(cookies, token, config.cookie_secure);
            let notice = "Passwort geändert. Andere Geräte sind abgemeldet.".to_string();
            account_page(pool, config, user.0, org, None, Some(notice), None).await
        }
        Err(e @ AppError::BadRequest(_)) => {
            account_page(pool, config, user.0, org, None, None, Some(&e)).await
        }
        Err(e) => Err(e),
    }
}

#[derive(FromForm)]
struct KeyForm<'r> {
    name: &'r str,
    #[field(default = Vec::new())]
    permissions: Vec<&'r str>,
    expires_days: Option<u32>,
}

#[post("/organizations/<org>/account/api-keys", data = "<form>")]
async fn create_key(
    user: CurrentUser,
    org: &str,
    form: Form<KeyForm<'_>>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    organizations::membership(pool, &user.0.id, org).await?;
    let permissions: Vec<Permission> = form
        .permissions
        .iter()
        .filter_map(|p| p.parse().ok())
        .collect();
    let expires = form
        .expires_days
        .filter(|d| (1..=3650).contains(d))
        .map(|d| now() + d as i64 * 86_400);
    match api_key::create(pool, &user.0.id, form.name, &permissions, expires).await {
        Ok((_, key)) => account_page(pool, config, user.0, org, Some(key), None, None).await,
        Err(e @ AppError::BadRequest(_)) => {
            account_page(pool, config, user.0, org, None, None, Some(&e)).await
        }
        Err(e) => Err(e),
    }
}

#[post("/organizations/<org>/account/api-keys/<id>/delete")]
async fn delete_key(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    api_key::delete(pool, &user.0.id, id).await?;
    Ok(Redirect::to(format!("/organizations/{org}/account")))
}

// ------------------------------------------------------------------ Organisation

async fn settings_page(
    pool: &SqlitePool,
    user: User,
    org: &str,
    new_secret: Option<(String, String)>,
    error: Option<&AppError>,
) -> AppResult<Page> {
    let shell = shell(pool, user, org, "settings", "").await?;
    let members = organizations::members(pool, org)
        .await?
        .into_iter()
        .map(|(_, username, role, since)| MemberRow {
            username,
            role,
            since,
        })
        .collect();
    let mut hooks = Vec::new();
    if shell.can_manage() {
        for (hook, events) in webhooks::list(pool, org).await? {
            let deliveries = webhooks::deliveries(pool, &hook.id, 10).await?;
            hooks.push(HookRow {
                hook,
                events,
                deliveries,
            });
        }
    }
    let page = render(&SettingsPage {
        shell,
        members,
        hooks,
        new_secret,
        error: error.map(AppError::public_message),
    })?;
    Ok(match error {
        Some(e) => Page::Status(Custom(e.status(), page)),
        None => Page::Html(page),
    })
}

/// Rolle prüfen: Besitzer oder Admin.
async fn require_manager(pool: &SqlitePool, user: &User, org: &str) -> AppResult<Role> {
    let (_, role) = organizations::membership(pool, &user.id, org).await?;
    if !organizations::can_manage(role) {
        return Err(AppError::Forbidden(
            "Nur Besitzer und Admins dürfen das.".into(),
        ));
    }
    Ok(role)
}

#[get("/organizations/<org>/settings")]
async fn settings(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Page> {
    settings_page(pool, user.0, org, None, None).await
}

#[derive(FromForm)]
struct NameForm<'r> {
    name: &'r str,
}

#[post("/organizations/<org>/settings/name", data = "<form>")]
async fn rename_org(
    user: CurrentUser,
    org: &str,
    form: Form<NameForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Page> {
    require_manager(pool, &user.0, org).await?;
    match organizations::rename(pool, org, form.name).await {
        Ok(_) => Ok(Page::Redirect(Redirect::to(format!(
            "/organizations/{org}/settings"
        )))),
        Err(e @ AppError::BadRequest(_)) => settings_page(pool, user.0, org, None, Some(&e)).await,
        Err(e) => Err(e),
    }
}

#[post("/organizations/<org>/settings/delete")]
async fn delete_org(user: CurrentUser, org: &str, pool: &State<SqlitePool>) -> AppResult<Redirect> {
    if require_manager(pool, &user.0, org).await? != Role::Owner {
        return Err(AppError::Forbidden(
            "Nur der Besitzer darf die Organisation löschen.".into(),
        ));
    }
    organizations::delete(pool, org).await?;
    Ok(Redirect::to("/"))
}

#[derive(FromForm)]
struct WebhookForm<'r> {
    name: &'r str,
    url: &'r str,
    #[field(default = "")]
    secret: &'r str,
    #[field(default = Vec::new())]
    events: Vec<&'r str>,
}

#[post("/organizations/<org>/webhooks", data = "<form>")]
async fn create_webhook(
    user: CurrentUser,
    org: &str,
    form: Form<WebhookForm<'_>>,
    pool: &State<SqlitePool>,
) -> AppResult<Page> {
    require_manager(pool, &user.0, org).await?;
    let events: Vec<WebhookEvent> = form.events.iter().filter_map(|e| e.parse().ok()).collect();
    let secret = Some(form.secret).filter(|s| !s.trim().is_empty());
    match webhooks::create(pool, org, &user.0.id, form.name, form.url, secret, &events).await {
        Ok((hook, secret)) => {
            settings_page(pool, user.0, org, Some((hook.name, secret)), None).await
        }
        Err(e @ AppError::BadRequest(_)) => settings_page(pool, user.0, org, None, Some(&e)).await,
        Err(e) => Err(e),
    }
}

#[post("/organizations/<org>/webhooks/<id>/toggle")]
async fn toggle_webhook(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    require_manager(pool, &user.0, org).await?;
    let hook = webhooks::get(pool, org, id).await?;
    webhooks::set_enabled(pool, org, id, !hook.enabled).await?;
    Ok(Redirect::to(format!("/organizations/{org}/settings")))
}

#[post("/organizations/<org>/webhooks/<id>/delete")]
async fn delete_webhook(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    require_manager(pool, &user.0, org).await?;
    webhooks::delete(pool, org, id).await?;
    Ok(Redirect::to(format!("/organizations/{org}/settings")))
}

// ------------------------------------------------------------------ E-Mail-Eingang

#[derive(FromForm)]
struct MailForm<'r> {
    #[field(default = "")]
    name: &'r str,
    organization_id: &'r str,
    host: &'r str,
    #[field(default = 993)]
    port: i64,
    #[field(default = "tls")]
    security: &'r str,
    username: &'r str,
    #[field(default = "")]
    password: &'r str,
    #[field(default = "INBOX")]
    folder: &'r str,
    #[field(default = false)]
    mark_seen: bool,
    #[field(default = "")]
    tags: &'r str,
    #[field(default = false)]
    enabled: bool,
    #[field(default = 5)]
    interval_minutes: i64,
}

impl MailForm<'_> {
    fn input(&self) -> crate::services::mail::AccountInput {
        crate::services::mail::AccountInput {
            name: self.name.into(),
            organization_id: self.organization_id.into(),
            host: self.host.into(),
            port: self.port,
            security: self.security.into(),
            username: self.username.into(),
            password: self.password.into(),
            folder: self.folder.into(),
            mark_seen: self.mark_seen,
            tags: vec![self.tags.to_string()],
            enabled: self.enabled,
            interval_minutes: self.interval_minutes,
        }
    }
}

#[post("/organizations/<org>/account/mail", data = "<form>")]
async fn create_mail(
    user: CurrentUser,
    org: &str,
    form: Form<MailForm<'_>>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    organizations::membership(pool, &user.0.id, org).await?;
    organizations::membership(pool, &user.0.id, form.organization_id).await?;
    match crate::services::mail::create(pool, &user.0.id, &form.input()).await {
        Ok(_) => Ok(Page::Redirect(Redirect::to(format!(
            "/organizations/{org}/account#mail"
        )))),
        Err(e @ AppError::BadRequest(_)) => {
            account_page(pool, config, user.0, org, None, None, Some(&e)).await
        }
        Err(e) => Err(e),
    }
}

#[post("/organizations/<org>/account/mail/<id>", data = "<form>")]
async fn update_mail(
    user: CurrentUser,
    org: &str,
    id: &str,
    form: Form<MailForm<'_>>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    organizations::membership(pool, &user.0.id, org).await?;
    organizations::membership(pool, &user.0.id, form.organization_id).await?;
    match crate::services::mail::update(pool, &user.0.id, id, &form.input()).await {
        Ok(_) => Ok(Page::Redirect(Redirect::to(format!(
            "/organizations/{org}/account#mail-{id}"
        )))),
        Err(e @ AppError::BadRequest(_)) => {
            account_page(pool, config, user.0, org, None, None, Some(&e)).await
        }
        Err(e) => Err(e),
    }
}

#[post("/organizations/<org>/account/mail/<id>/delete")]
async fn delete_mail(
    user: CurrentUser,
    org: &str,
    id: &str,
    pool: &State<SqlitePool>,
) -> AppResult<Redirect> {
    crate::services::mail::delete(pool, &user.0.id, id).await?;
    Ok(Redirect::to(format!("/organizations/{org}/account#mail")))
}

/// Jetzt abholen (statt auf den nächsten Durchlauf zu warten).
#[post("/mail/<id>/fetch")]
async fn fetch_mail(
    user: CurrentUser,
    id: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<rocket::serde::json::Value> {
    let account = crate::services::mail::get(pool, &user.0.id, id).await?;
    let ai = crate::ai::AiConfig::load(pool, config).await;
    let report = crate::services::mail::run(pool, &ai, &account)
        .await
        .map_err(AppError::bad)?;
    Ok(rocket::serde::json::json!({ "report": report }))
}

/// Verbindung testen: anmelden und den Ordner öffnen, nichts abholen.
#[post("/mail/<id>/test")]
async fn test_mail(
    user: CurrentUser,
    id: &str,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<rocket::serde::json::Value> {
    let account = crate::services::mail::get(pool, &user.0.id, id).await?;
    let ai = crate::ai::AiConfig::load(pool, config).await;
    crate::services::mail::fetch(pool, &ai, &account, true)
        .await
        .map_err(AppError::bad)?;
    Ok(rocket::serde::json::json!({ "ok": true }))
}

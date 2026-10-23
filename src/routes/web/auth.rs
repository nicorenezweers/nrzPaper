//! Anmelden, Registrieren, Abmelden.

use super::{safe_next, Page};
use crate::auth::guards::{MaybeUser, UserAgent};
use crate::auth::session;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::services::{organizations, users};
use crate::templates::{render, LoginPage, RegisterPage};
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket::response::status::Custom;
use rocket::response::Redirect;
use rocket::{get, post, routes, FromForm, Route, State};
use sqlx::SqlitePool;
use std::time::Duration;

pub fn routes() -> Vec<Route> {
    routes![index, login_page, login, logout, register_page, register]
}

/// Start: angemeldet → erste Organisation (oder anlegen), sonst Anmeldung.
#[get("/")]
async fn index(user: MaybeUser, pool: &State<SqlitePool>) -> AppResult<Redirect> {
    let Some(user) = user.0 else {
        return Ok(Redirect::to(if users::count(pool).await? == 0 {
            "/register"
        } else {
            "/login"
        }));
    };
    Ok(
        match organizations::list_for_user(pool, &user.id).await?.first() {
            Some((org, _)) => Redirect::to(format!("/organizations/{}", org.id)),
            None => Redirect::to("/organizations/new"),
        },
    )
}

#[get("/login?<next>")]
async fn login_page(
    user: MaybeUser,
    next: Option<&str>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    if user.0.is_some() {
        return Ok(Page::Redirect(Redirect::to(safe_next(next))));
    }
    if users::count(pool).await? == 0 {
        return Ok(Page::Redirect(Redirect::to("/register")));
    }
    let page = LoginPage {
        error: None,
        next: safe_next(next),
        username: String::new(),
        can_register: users::registration_allowed(pool, config).await?,
    };
    Ok(Page::Html(render(&page)?))
}

#[derive(FromForm)]
struct LoginForm<'r> {
    username: &'r str,
    password: &'r str,
    next: Option<&'r str>,
}

#[post("/login", data = "<form>")]
async fn login(
    form: Form<LoginForm<'_>>,
    cookies: &CookieJar<'_>,
    ua: UserAgent,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    match users::authenticate(pool, form.username, form.password).await? {
        Some(user) => {
            let token = session::create(pool, &user.id, &ua.0).await?;
            session::set_cookie(cookies, token, config.cookie_secure);
            tracing::info!(user = %user.username, "Anmeldung");
            Ok(Page::Redirect(Redirect::to(safe_next(form.next))))
        }
        None => {
            // Bremst Durchprobieren von Passwörtern.
            tokio::time::sleep(Duration::from_millis(400)).await;
            tracing::warn!(username = form.username, "Anmeldung fehlgeschlagen");
            let page = LoginPage {
                error: Some("Benutzername oder Passwort stimmt nicht.".into()),
                next: safe_next(form.next),
                username: form.username.to_string(),
                can_register: users::registration_allowed(pool, config).await?,
            };
            Ok(Page::Status(Custom(Status::Unauthorized, render(&page)?)))
        }
    }
}

#[post("/logout")]
async fn logout(cookies: &CookieJar<'_>, pool: &State<SqlitePool>) -> AppResult<Redirect> {
    if let Some(token) = session::token_from(cookies) {
        session::destroy(pool, &token).await?;
    }
    session::clear_cookie(cookies);
    Ok(Redirect::to("/login"))
}

#[get("/register")]
async fn register_page(
    user: MaybeUser,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    if user.0.is_some() {
        return Ok(Page::Redirect(Redirect::to("/")));
    }
    if !users::registration_allowed(pool, config).await? {
        return Err(AppError::forbidden(
            "Die Registrierung ist geschlossen – ein Administrator muss dein Konto anlegen.",
        ));
    }
    let page = RegisterPage {
        error: None,
        username: String::new(),
        first_user: users::count(pool).await? == 0,
    };
    Ok(Page::Html(render(&page)?))
}

#[derive(FromForm)]
struct RegisterForm<'r> {
    username: &'r str,
    password: &'r str,
}

#[post("/register", data = "<form>")]
async fn register(
    form: Form<RegisterForm<'_>>,
    cookies: &CookieJar<'_>,
    ua: UserAgent,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    if !users::registration_allowed(pool, config).await? {
        return Err(AppError::forbidden("Die Registrierung ist geschlossen."));
    }
    match users::create(pool, form.username, form.password, "").await {
        Ok(user) => {
            let token = session::create(pool, &user.id, &ua.0).await?;
            session::set_cookie(cookies, token, config.cookie_secure);
            tracing::info!(user = %user.username, admin = user.is_admin, "Konto angelegt");
            Ok(Page::Redirect(Redirect::to("/organizations/new")))
        }
        Err(e @ (AppError::BadRequest(_) | AppError::Conflict(_))) => {
            let page = RegisterPage {
                error: Some(e.public_message()),
                username: form.username.to_string(),
                first_user: users::count(pool).await? == 0,
            };
            Ok(Page::Status(Custom(e.status(), render(&page)?)))
        }
        Err(e) => Err(e),
    }
}

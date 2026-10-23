//! Request Guards. Routen verlangen damit eine Anmeldung:
//!
//! * [`CurrentUser`] – angemeldet über das Sitzungs-Cookie (Oberfläche).
//! * [`AdminUser`] – zusätzlich Administrator der Instanz.
//! * [`ApiUser`] – API-Key (`Authorization: Bearer np_…`) oder Sitzung; Rechte per [`ApiUser::require`].
//!
//! Schutz gegen CSRF: Das Cookie ist `SameSite=Lax` (fremde Seiten können kein POST mit
//! Cookie auslösen). Zusätzlich werden ändernde Anfragen abgewiesen, deren `Origin` nicht
//! zum eigenen Host passt.

use crate::auth::{api_key, session};
use crate::db::models::{ApiKey, Permission, User};
use crate::error::{AppError, AppResult};
use crate::services::users;
use rocket::http::{Method, Status};
use rocket::request::{FromRequest, Outcome, Request};
use sqlx::SqlitePool;

/// Benutzer der Sitzung – je Anfrage nur einmal aus der Datenbank geladen.
async fn session_user(req: &Request<'_>) -> Option<User> {
    req.local_cache_async(async {
        let pool = req.rocket().state::<SqlitePool>()?;
        let token = session::token_from(req.cookies())?;
        session::lookup(pool, &token).await.ok().flatten()
    })
    .await
    .clone()
}

/// Ändernde Anfrage von einer fremden Seite?
pub fn is_cross_origin(req: &Request<'_>) -> bool {
    if matches!(req.method(), Method::Get | Method::Head | Method::Options) {
        return false;
    }
    let Some(origin) = req.headers().get_one("Origin") else { return false };
    let origin_host = origin.split_once("://").map(|(_, h)| h).unwrap_or("");
    let own = [req.headers().get_one("X-Forwarded-Host"), req.headers().get_one("Host")];
    !own.iter().flatten().any(|h| h.eq_ignore_ascii_case(origin_host))
}

pub struct CurrentUser(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CurrentUser {
    type Error = AppError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if is_cross_origin(req) {
            return Outcome::Error((Status::Forbidden, AppError::forbidden("Anfrage von einer fremden Seite abgewiesen.")));
        }
        match session_user(req).await {
            Some(user) => Outcome::Success(CurrentUser(user)),
            None => Outcome::Error((Status::Unauthorized, AppError::Unauthorized)),
        }
    }
}

/// Angemeldet, aber optional – für Seiten wie die Anmeldung selbst.
pub struct MaybeUser(pub Option<User>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MaybeUser {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(MaybeUser(session_user(req).await))
    }
}

pub struct AdminUser(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = AppError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match CurrentUser::from_request(req).await {
            Outcome::Success(CurrentUser(u)) if u.is_admin => Outcome::Success(AdminUser(u)),
            Outcome::Success(_) => Outcome::Error((Status::Forbidden, AppError::forbidden("Nur für Administratoren."))),
            Outcome::Error(e) => Outcome::Error(e),
            Outcome::Forward(s) => Outcome::Forward(s),
        }
    }
}

/// Zugriff auf die API. Mit API-Key gelten dessen Berechtigungen, mit Sitzung alle.
pub struct ApiUser {
    pub user: User,
    pub key: Option<ApiKey>,
}

impl ApiUser {
    pub fn require(&self, permission: Permission) -> AppResult<()> {
        match &self.key {
            Some(k) if !k.permissions.0.contains(&permission) => {
                Err(AppError::forbidden(format!("Dem API-Key fehlt die Berechtigung „{permission}“.")))
            }
            _ => Ok(()),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiUser {
    type Error = AppError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let unauthorized = || Outcome::Error((Status::Unauthorized, AppError::Unauthorized));
        let Some(pool) = req.rocket().state::<SqlitePool>() else { return unauthorized() };

        if let Some(auth) = req.headers().get_one("Authorization") {
            let Some(key) = auth.strip_prefix("Bearer ").map(str::trim) else { return unauthorized() };
            return match api_key::lookup(pool, key).await {
                Ok(Some(k)) => match users::by_id(pool, &k.user_id).await {
                    Ok(Some(user)) => Outcome::Success(ApiUser { user, key: Some(k) }),
                    _ => unauthorized(),
                },
                Ok(None) => unauthorized(),
                Err(e) => Outcome::Error((Status::InternalServerError, e)),
            };
        }
        match CurrentUser::from_request(req).await {
            Outcome::Success(CurrentUser(user)) => Outcome::Success(ApiUser { user, key: None }),
            Outcome::Error(e) => Outcome::Error(e),
            Outcome::Forward(s) => Outcome::Forward(s),
        }
    }
}

/// Browser-Kennung für die Sitzungsliste.
pub struct UserAgent(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(UserAgent(req.headers().get_one("User-Agent").unwrap_or_default().to_string()))
    }
}

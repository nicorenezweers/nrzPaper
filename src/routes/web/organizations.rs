//! Organisationen anlegen und öffnen.

use super::Page;
use crate::auth::guards::CurrentUser;
use crate::error::{AppError, AppResult};
use crate::services::organizations;
use crate::templates::{render, NewOrgPage, OrgHomePage, Shell};
use rocket::form::Form;
use rocket::response::status::Custom;
use rocket::response::Redirect;
use rocket::{get, post, routes, FromForm, Route, State};
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![new_page, create, home]
}

/// Rahmen für eine Seite innerhalb einer Organisation (404, wenn kein Mitglied).
pub async fn shell(pool: &SqlitePool, user: crate::db::models::User, org_id: &str, active: &'static str) -> AppResult<Shell> {
    let (org, role) = organizations::membership(pool, &user.id, org_id).await?;
    let orgs = organizations::list_for_user(pool, &user.id).await?;
    Ok(Shell { user, org, role, orgs, active })
}

#[get("/organizations/new")]
async fn new_page(user: CurrentUser) -> AppResult<Page> {
    Ok(Page::Html(render(&NewOrgPage { error: None, org_name: String::new(), username: user.0.username })?))
}

#[derive(FromForm)]
struct NewOrgForm<'r> {
    name: &'r str,
}

#[post("/organizations/new", data = "<form>")]
async fn create(user: CurrentUser, form: Form<NewOrgForm<'_>>, pool: &State<SqlitePool>) -> AppResult<Page> {
    match organizations::create(pool, &user.0.id, form.name).await {
        Ok(org) => Ok(Page::Redirect(Redirect::to(format!("/organizations/{}", org.id)))),
        Err(e @ AppError::BadRequest(_)) => {
            let page = NewOrgPage { error: Some(e.public_message()), org_name: form.name.to_string(), username: user.0.username };
            Ok(Page::Status(Custom(e.status(), render(&page)?)))
        }
        Err(e) => Err(e),
    }
}

#[get("/organizations/<id>")]
async fn home(user: CurrentUser, id: &str, pool: &State<SqlitePool>) -> AppResult<Page> {
    let shell = shell(pool, user.0, id, "").await?;
    Ok(Page::Html(render(&OrgHomePage { shell })?))
}

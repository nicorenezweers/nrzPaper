//! Organisationen anlegen und öffnen.

use super::Page;
use crate::auth::guards::CurrentUser;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::services::organizations;
use crate::templates::{render, NewOrgPage};
use rocket::form::Form;
use rocket::response::status::Custom;
use rocket::response::Redirect;
use rocket::{get, post, routes, FromForm, Route, State};
use sqlx::SqlitePool;

pub fn routes() -> Vec<Route> {
    routes![new_page, create]
}

#[get("/organizations/new")]
async fn new_page(user: CurrentUser) -> AppResult<Page> {
    Ok(Page::Html(render(&NewOrgPage {
        error: None,
        org_name: String::new(),
        username: user.0.username,
    })?))
}

#[derive(FromForm)]
struct NewOrgForm<'r> {
    name: &'r str,
}

#[post("/organizations/new", data = "<form>")]
async fn create(
    user: CurrentUser,
    form: Form<NewOrgForm<'_>>,
    pool: &State<SqlitePool>,
    config: &State<Config>,
) -> AppResult<Page> {
    match organizations::create(pool, &user.0.id, form.name).await {
        Ok(org) => {
            crate::ingestion::create_org_dir(config, &org.id).await;
            Ok(Page::Redirect(Redirect::to(format!(
                "/organizations/{}",
                org.id
            ))))
        }
        Err(e @ AppError::BadRequest(_)) => {
            let page = NewOrgPage {
                error: Some(e.public_message()),
                org_name: form.name.to_string(),
                username: user.0.username,
            };
            Ok(Page::Status(Custom(e.status(), render(&page)?)))
        }
        Err(e) => Err(e),
    }
}

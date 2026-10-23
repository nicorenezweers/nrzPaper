//! Askama-Templates (Ordner `templates/`) – zur Übersetzungszeit geprüft.

use crate::db::models::{Organization, Role, User};
use crate::error::AppResult;
use askama::Template;
use rocket::response::content::RawHtml;

pub fn render<T: Template>(page: &T) -> AppResult<RawHtml<String>> {
    Ok(RawHtml(page.render()?))
}

/// Rahmen der App: Benutzer, aktuelle Organisation, Organisationsliste, aktiver Menüpunkt.
pub struct Shell {
    pub user: User,
    pub org: Organization,
    pub role: Role,
    pub orgs: Vec<(Organization, Role)>,
    pub active: &'static str,
}

impl Shell {
    pub fn org_initial(&self) -> String {
        self.org.name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default()
    }

    pub fn role_label(&self) -> &'static str {
        match self.role {
            Role::Owner => "Besitzer",
            Role::Admin => "Admin",
            Role::Member => "Mitglied",
        }
    }

    /// Menüpunkte: (Pfad unterhalb der Organisation, Bezeichnung, schon verfügbar)
    pub fn nav(&self) -> Vec<(&'static str, &'static str, bool)> {
        vec![
            ("", "Dokumente", true),
            ("tags", "Tags", false),
            ("tagging-rules", "Tagging-Regeln", false),
            ("properties", "Eigenschaften", false),
            ("trash", "Papierkorb", false),
            ("settings", "Einstellungen", false),
        ]
    }
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorPage<'a> {
    pub status: u16,
    pub title: &'a str,
    pub message: &'a str,
}

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginPage {
    pub error: Option<String>,
    pub next: String,
    pub username: String,
    pub can_register: bool,
}

#[derive(Template)]
#[template(path = "auth/register.html")]
pub struct RegisterPage {
    pub error: Option<String>,
    pub username: String,
    pub first_user: bool,
}

#[derive(Template)]
#[template(path = "orgs/new.html")]
pub struct NewOrgPage {
    pub error: Option<String>,
    pub org_name: String,
    pub username: String,
}

#[derive(Template)]
#[template(path = "orgs/home.html")]
pub struct OrgHomePage {
    pub shell: Shell,
}

//! nrzPaper – Oberfläche (Dioxus 0.7, WebAssembly) mit Komponenten von rust-ui.
//! Spricht nur mit der JSON-API unter `/api/v1` (Sitzungs-Cookie).

mod api;
mod components;
mod i18n;
mod models;
mod pages;
mod state;
mod ui;
mod util;

use components::overlay::{ConfirmHost, Toaster};
use dioxus::prelude::*;
use pages::auth::{Login, Register, Setup};
use pages::document::DocumentPage;
use pages::documents::{Documents, FolderView};
use pages::home::{Home, NewOrg, NotFound};
use pages::inbox::Inbox;
use pages::layout::{AppLayout, OrgLayout};
use pages::scan::Scan;
use pages::settings::Settings;
use pages::share::Share;
use pages::tags::Tags;
use pages::reminders::RemindersPage;
use pages::trash::Trash;

static CSS: Asset = asset!("/assets/tailwind.css");
static ICON: Asset = asset!("/assets/icon.png");
static FAVICON: Asset = asset!("/assets/favicon-32.png");

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[route("/login?:next")]
    Login { next: String },
    #[route("/setup")]
    Setup {},
    #[route("/register")]
    Register {},
    #[route("/share/:token")]
    Share { token: String },
    #[layout(AppLayout)]
        #[route("/")]
        Home {},
        #[route("/organizations/new")]
        NewOrg {},
        #[layout(OrgLayout)]
            #[route("/organizations/:org?:q")]
            Documents { org: String, q: String },
            #[route("/organizations/:org/folders/:folder?:q")]
            FolderView { org: String, folder: String, q: String },
            #[route("/organizations/:org/documents/:id")]
            DocumentPage { org: String, id: String },
            #[route("/organizations/:org/tags")]
            Tags { org: String },
            #[route("/organizations/:org/trash")]
            Trash { org: String },
            #[route("/organizations/:org/reminders")]
            RemindersPage { org: String },
            #[route("/organizations/:org/scan")]
            Scan { org: String },
            #[route("/organizations/:org/inbox?:item")]
            Inbox { org: String, item: String },
            #[route("/organizations/:org/settings/:section")]
            Settings { org: String, section: String },
        #[end_layout]
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

impl Route {
    /// Organisation in der Adresse, falls es eine gibt.
    pub fn org(&self) -> Option<&str> {
        match self {
            Route::Documents { org, .. }
            | Route::FolderView { org, .. }
            | Route::DocumentPage { org, .. }
            | Route::Tags { org }
            | Route::Trash { org }
            | Route::RemindersPage { org }
            | Route::Scan { org }
            | Route::Inbox { org, .. }
            | Route::Settings { org, .. } => Some(org),
            _ => None,
        }
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut ready = use_signal(|| false);
    use_future(move || async move {
        i18n::init().await;
        ready.set(true);
    });
    rsx! {
        document::Stylesheet { href: CSS }
        // PDF-Viewer (Seiten, Textebene, Treffer) – vom Server ausgeliefert
        document::Stylesheet { href: concat!("/static/viewer.css?v=", env!("CARGO_PKG_VERSION")) }
        document::Link { rel: "icon", r#type: "image/png", href: FAVICON }
        document::Link { rel: "apple-touch-icon", href: ICON }
        document::Meta { name: "theme-color", content: "#1f1f1f" }
        if ready() {
            Router::<Route> {}
        }
        ConfirmHost {}
        Toaster {}
    }
}

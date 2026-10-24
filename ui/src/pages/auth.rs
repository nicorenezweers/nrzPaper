//! Anmelden, Ersteinrichtung (erstes Konto wird Administrator) und Registrierung.

use crate::api;
use crate::components::common::*;
use crate::models::AuthStatus;
use crate::state;
use crate::Route;
use dioxus::prelude::*;

/// Gemeinsamer Rahmen der Anmeldeseiten: Logo, Karte, Fusszeile.
#[component]
fn AuthCard(#[props(into)] title: String, #[props(into)] subtitle: String, children: Element) -> Element {
    rsx! {
        div { class: "flex min-h-screen items-center justify-center bg-background p-4",
            div { class: "w-full max-w-sm",
                div { class: "mb-6 flex flex-col items-center gap-3 text-center",
                    img { src: asset!("/assets/icon.png"), class: "size-12 rounded-xl", alt: "" }
                    div {
                        h1 { class: "text-xl font-semibold tracking-tight", "{title}" }
                        p { class: "mt-1 text-sm text-muted-foreground", "{subtitle}" }
                    }
                }
                div { class: "rounded-xl border bg-card p-6 shadow-sm", {children} }
                p { class: "mt-6 text-center text-xs text-muted-foreground", "nrzPaper – Dokumente, verschlüsselt in einer Datei" }
            }
        }
    }
}

fn safe_next(next: &str) -> String {
    if next.starts_with('/') && !next.starts_with("//") && !next.contains('\\') { next.to_string() } else { "/".into() }
}

#[component]
pub fn Login(next: String) -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let status = use_resource(|| api::get::<AuthStatus>("/auth/status"));
    let nav = use_navigator();
    if let Some(Ok(s)) = &*status.read() {
        if s.needs_setup {
            nav.replace(Route::Setup {});
        }
    }
    let can_register = status.read().as_ref().and_then(|r| r.as_ref().ok()).is_some_and(|s| s.registration_open);
    let target = safe_next(&next);
    let submit = move |e: FormEvent| {
        e.prevent_default();
        let target = target.clone();
        spawn(async move {
            busy.set(true);
            error.set(None);
            let res: api::ApiResult<serde_json::Value> = api::post("/auth/login", &serde_json::json!({ "username": username(), "password": password() })).await;
            busy.set(false);
            match res {
                Ok(_) => {
                    state::load_me().await;
                    // Ganze Seite laden, damit alles frisch startet
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().assign(&target);
                    }
                }
                Err(e) if e.status == 401 => error.set(Some("Benutzername oder Passwort stimmt nicht.".into())),
                Err(e) => error.set(Some(e.message)),
            }
        });
    };
    rsx! {
        AuthCard { title: "Anmelden", subtitle: "Willkommen zurück bei nrzPaper",
            form { class: "grid gap-4", onsubmit: submit,
                if let Some(e) = error() { Callout { text: e, error: true } }
                Field { label: "Benutzername",
                    input { class: INPUT, autocomplete: "username", autofocus: true, required: true, value: "{username}", oninput: move |e| username.set(e.value()) }
                }
                Field { label: "Passwort",
                    input { class: INPUT, r#type: "password", autocomplete: "current-password", required: true, value: "{password}", oninput: move |e| password.set(e.value()) }
                }
                button { class: "{btn(BTN_PRIMARY)} w-full", disabled: busy(), r#type: "submit",
                    if busy() { icons::LoaderCircle { class: "size-4 animate-spin" } }
                    "Anmelden"
                }
                if can_register {
                    p { class: "text-center text-sm text-muted-foreground",
                        "Noch kein Konto? "
                        Link { class: "font-medium text-foreground underline-offset-4 hover:underline", to: Route::Register {}, "Registrieren" }
                    }
                }
            }
        }
    }
}

#[component]
fn RegisterForm(first: bool) -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let submit = move |e: FormEvent| {
        e.prevent_default();
        if password() != confirm() {
            error.set(Some("Die Passwörter stimmen nicht überein.".into()));
            return;
        }
        spawn(async move {
            busy.set(true);
            error.set(None);
            let res: api::ApiResult<serde_json::Value> = api::post("/auth/register", &serde_json::json!({ "username": username(), "password": password() })).await;
            busy.set(false);
            match res {
                Ok(_) => {
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().assign("/organizations/new");
                    }
                }
                Err(e) => error.set(Some(e.message)),
            }
        });
    };
    rsx! {
        form { class: "grid gap-4", onsubmit: submit,
            if first {
                Callout { text: "Das erste Konto wird Administrator dieser Instanz." }
            }
            if let Some(e) = error() { Callout { text: e, error: true } }
            Field { label: "Benutzername", hint: "3–32 Zeichen: Buchstaben, Zahlen, Punkt, Minus, Unterstrich",
                input { class: INPUT, autocomplete: "username", autofocus: true, required: true, value: "{username}", oninput: move |e| username.set(e.value()) }
            }
            Field { label: "Passwort", hint: "Mindestens 8 Zeichen",
                input { class: INPUT, r#type: "password", autocomplete: "new-password", required: true, minlength: 8, value: "{password}", oninput: move |e| password.set(e.value()) }
            }
            Field { label: "Passwort wiederholen",
                input { class: INPUT, r#type: "password", autocomplete: "new-password", required: true, value: "{confirm}", oninput: move |e| confirm.set(e.value()) }
            }
            button { class: "{btn(BTN_PRIMARY)} w-full", disabled: busy(), r#type: "submit",
                if busy() { icons::LoaderCircle { class: "size-4 animate-spin" } }
                if first { "Einrichten" } else { "Konto anlegen" }
            }
            if !first {
                p { class: "text-center text-sm text-muted-foreground",
                    "Schon ein Konto? "
                    Link { class: "font-medium text-foreground underline-offset-4 hover:underline", to: Route::Login { next: String::new() }, "Anmelden" }
                }
            }
        }
    }
}

#[component]
pub fn Setup() -> Element {
    rsx! {
        AuthCard { title: "nrzPaper einrichten", subtitle: "Lege das erste Konto an.",
            RegisterForm { first: true }
        }
    }
}

#[component]
pub fn Register() -> Element {
    rsx! {
        AuthCard { title: "Registrieren", subtitle: "Neues Konto bei nrzPaper",
            RegisterForm { first: false }
        }
    }
}

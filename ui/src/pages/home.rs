//! Startseite (weiter zur ersten Organisation), neue Organisation, unbekannte Adresse.

use crate::api;
use crate::components::common::*;
use crate::state::{self, ME};
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let nav = use_navigator();
    let me = ME.read().clone();
    match me.and_then(|m| m.organizations.first().cloned()) {
        Some(org) => {
            nav.replace(Route::Documents { org: org.id, q: String::new() });
            rsx! {}
        }
        None => {
            nav.replace(Route::NewOrg {});
            rsx! {}
        }
    }
}

#[component]
pub fn NewOrg() -> Element {
    let nav = use_navigator();
    let mut name = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let first = ME.read().as_ref().is_some_and(|m| m.organizations.is_empty());
    let submit = move |e: FormEvent| {
        e.prevent_default();
        spawn(async move {
            busy.set(true);
            let res: api::ApiResult<serde_json::Value> = api::post("/organizations", &serde_json::json!({ "name": name() })).await;
            busy.set(false);
            match res {
                Ok(v) => {
                    state::load_me().await;
                    let id = v["organization"]["id"].as_str().unwrap_or_default().to_string();
                    nav.replace(Route::Documents { org: id, q: String::new() });
                }
                Err(e) => error.set(Some(e.message)),
            }
        });
    };
    rsx! {
        div { class: "flex min-h-screen items-center justify-center p-4",
            div { class: "w-full max-w-md",
                div { class: "mb-6 text-center",
                    img { src: asset!("/assets/icon.png"), class: "mx-auto size-12 rounded-xl", alt: "" }
                    h1 { class: "mt-3 text-xl font-semibold tracking-tight", if first { "Erste Organisation" } else { "Neue Organisation" } }
                    p { class: "mt-1 text-sm text-muted-foreground", "Eine Organisation trennt Dokumente, Tags und Mitglieder – z. B. „Privat“ und „Gewerbe“." }
                }
                form { class: "grid gap-4 rounded-xl border bg-card p-6 shadow-sm", onsubmit: submit,
                    if let Some(e) = error() { Callout { text: e, error: true } }
                    Field { label: "Name",
                        input { class: INPUT, autofocus: true, required: true, maxlength: 100, placeholder: "z. B. Privat", value: "{name}", oninput: move |e| name.set(e.value()) }
                    }
                    div { class: "flex justify-end gap-2",
                        if !first {
                            button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| { nav.go_back(); }, "Abbrechen" }
                        }
                        button { class: btn(BTN_PRIMARY), disabled: busy() || name().trim().is_empty(), r#type: "submit", "Anlegen" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        div { class: "grid min-h-screen place-items-center p-6",
            EmptyState { title: "Seite nicht gefunden", description: "Diese Adresse gibt es in nrzPaper nicht.",
                Link { class: btn(BTN_OUTLINE), to: Route::Home {}, "Zur Übersicht" }
            }
        }
    }
}

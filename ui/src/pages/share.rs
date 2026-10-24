//! Öffentliche Freigabe: ein Dokument ohne Anmeldung ansehen und herunterladen.

use crate::api;
use crate::components::common::*;
use crate::models::Document;
use crate::pages::document::Viewer;
use crate::util;
use dioxus::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct Shared {
    document: Document,
    expires_at: Option<i64>,
    file_url: String,
}

#[component]
pub fn Share(token: String) -> Element {
    let t = token.clone();
    let data = use_resource(use_reactive!(|t| async move { api::get::<Shared>(&format!("/share/{}", api::enc(&t))).await }));
    rsx! {
        div { class: "mx-auto flex min-h-screen max-w-6xl flex-col px-4 py-5 sm:px-6",
            match &*data.read() {
                None => rsx! { Loading {} },
                Some(Err(_)) => rsx! {
                    div { class: "grid flex-1 place-items-center",
                        EmptyState { title: "Link ungültig", description: "Diese Freigabe gibt es nicht mehr – sie ist abgelaufen oder wurde widerrufen.", icon: rsx! { icons::Link {} } }
                    }
                },
                Some(Ok(s)) => {
                    let mut doc = s.document.clone();
                    doc.id = format!("share-{token}");
                    rsx! {
                        header { class: "mb-4 flex flex-wrap items-center justify-between gap-4",
                            div { class: "flex min-w-0 items-center gap-3",
                                img { src: asset!("/assets/icon.png"), class: "size-9 rounded-lg", alt: "" }
                                div { class: "min-w-0",
                                    p { class: "text-xs uppercase tracking-wider text-muted-foreground", "Geteiltes Dokument · nrzPaper" }
                                    h1 { class: "truncate text-xl font-semibold tracking-tight", "{doc.name}" }
                                    p { class: "text-sm text-muted-foreground",
                                        "{doc.type_label()} · {util::size(doc.size)}"
                                        if let Some(n) = doc.page_count { " · {n} " if n == 1 { "Seite" } else { "Seiten" } }
                                        if let Some(e) = s.expires_at { " · Link gültig bis {util::date(e)}" }
                                    }
                                }
                            }
                            a { class: btn(BTN_PRIMARY), href: "{s.file_url}?download=true", icons::Download { class: "size-4" } "Herunterladen" }
                        }
                        Viewer { url: s.file_url.clone(), doc }
                    }
                }
            }
        }
    }
}

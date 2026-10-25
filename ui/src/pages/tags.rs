//! Tags: anlegen, bearbeiten (Name, Farbe, Beschreibung), löschen – mit eigenen Dialogen.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::Tag;
use crate::state::{self, OVERVIEW};
use crate::Route;
use dioxus::prelude::*;
use serde_json::json;
use crate::i18n::{tr, trf};

pub const PALETTE: &[&str] = &["#a1a1aa", "#ef4444", "#f97316", "#eab308", "#22c55e", "#14b8a6", "#3b82f6", "#8b5cf6", "#ec4899", "#78716c"];

#[component]
pub fn Tags(org: String) -> Element {
    let mut open = use_signal(|| false);
    let mut editing = use_signal(|| None::<Tag>);
    let tags: Vec<Tag> = OVERVIEW.read().clone().filter(|(o, _)| *o == org).map(|(_, v)| v.tags).unwrap_or_default();
    rsx! {
        div { class: "mx-auto max-w-4xl p-4 md:p-6",
            PageHeader { title: tr("Tags"), description: tr("Ordnen Dokumente. Ein Klick auf die Anzahl zeigt die Dokumente."),
                actions: rsx! {
                    button { class: btn(BTN_PRIMARY), onclick: move |_| { editing.set(None); open.set(true); }, icons::Plus { class: "size-4" } {tr("Neuer Tag")} }
                }
            }
            if tags.is_empty() {
                EmptyState { title: tr("Noch keine Tags"), description: tr("Tags fassen Dokumente zusammen – und Ordner fassen Tags zusammen."), icon: rsx! { icons::Tag {} } }
            } else {
                div { class: "divide-y overflow-hidden rounded-xl border bg-card",
                    for t in tags.iter().cloned() {
                        div { key: "{t.id}", class: "flex items-center gap-3 px-4 py-3",
                            span { class: "size-3 shrink-0 rounded-full", style: "background: {t.color}" }
                            div { class: "min-w-0 flex-1",
                                p { class: "truncate font-medium", "{t.name}" }
                                if !t.description.is_empty() { p { class: "truncate text-sm text-muted-foreground", "{t.description}" } }
                            }
                            Link { class: "text-sm text-muted-foreground hover:text-foreground",
                                to: Route::Documents { org: org.clone(), q: if t.name.contains(' ') { format!("tag:\"{}\"", t.name) } else { format!("tag:{}", t.name) } },
                                "{t.documents_count} " if t.documents_count == 1 { {tr("Dokument")} } else { {tr("Dokumente")} }
                            }
                            button { class: "{btn(BTN_GHOST)} {BTN_ICON}", title: tr("Bearbeiten"), onclick: { let t = t.clone(); move |_| { editing.set(Some(t.clone())); open.set(true); } }, icons::Pencil { class: "size-4" } }
                            button { class: "{btn(BTN_GHOST)} {BTN_ICON} text-muted-foreground hover:text-destructive", title: tr("Löschen"),
                                onclick: { let (org, t) = (org.clone(), t.clone()); move |_| {
                                    let (org, t) = (org.clone(), t.clone());
                                    spawn(async move {
                                        let ask = Ask::new(trf("Tag „{0}“ löschen?", &[&(t.name)]), trf("Er wird von {0} Dokument(en) entfernt. Die Dokumente bleiben.", &[&(t.documents_count)])).confirm(tr("Löschen")).destructive();
                                        if confirm(ask).await {
                                            match api::delete(&format!("/organizations/{org}/tags/{}", t.id)).await {
                                                Ok(()) => { toast(tr("Tag gelöscht.")); state::refresh(); }
                                                Err(e) => toast_error(e),
                                            }
                                        }
                                    });
                                } },
                                icons::Trash2 { class: "size-4" }
                            }
                        }
                    }
                }
            }
            TagDialog { open, org: org.clone(), tag: editing() }
        }
    }
}

#[component]
pub fn TagDialog(open: Signal<bool>, org: String, tag: Option<Tag>) -> Element {
    let mut name = use_signal(String::new);
    let mut color = use_signal(|| PALETTE[0].to_string());
    let mut description = use_signal(String::new);
    let init = tag.clone();
    use_effect(use_reactive!(|init| {
        if open() {
            name.set(init.as_ref().map(|t| t.name.clone()).unwrap_or_default());
            color.set(init.as_ref().map(|t| t.color.clone()).unwrap_or_else(|| PALETTE[0].to_string()));
            description.set(init.as_ref().map(|t| t.description.clone()).unwrap_or_default());
        }
    }));
    let id = tag.as_ref().map(|t| t.id.clone());
    let title = if tag.is_some() { tr("Tag bearbeiten") } else { tr("Neuer Tag") };
    rsx! {
        Modal { open, title, description: tr("Die Beschreibung hilft der KI, den Tag richtig zu vergeben."),
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    let (org, id) = (org.clone(), id.clone());
                    spawn(async move {
                        let body = json!({ "name": name(), "color": color(), "description": description() });
                        let res: api::ApiResult<serde_json::Value> = match &id {
                            Some(id) => api::put(&format!("/organizations/{org}/tags/{id}"), &body).await,
                            None => api::post(&format!("/organizations/{org}/tags"), &body).await,
                        };
                        match res {
                            Ok(_) => { toast(tr("Tag gespeichert.")); open.set(false); state::refresh(); }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                Field { label: tr("Name"), input { class: INPUT, required: true, maxlength: 64, value: "{name}", oninput: move |e| name.set(e.value()) } }
                Field { label: tr("Beschreibung"), hint: tr("z. B. „Rechnungen und Belege mit Betrag“"),
                    input { class: INPUT, value: "{description}", oninput: move |e| description.set(e.value()) }
                }
                div { class: "grid gap-1.5",
                    span { class: "text-sm font-medium", {tr("Farbe")} }
                    div { class: "flex flex-wrap gap-2",
                        for c in PALETTE.iter().copied() {
                            button { r#type: "button", class: if color() == c { "size-7 rounded-full ring-2 ring-foreground ring-offset-2 ring-offset-background" } else { "size-7 rounded-full" },
                                style: "background: {c}", title: "{c}", onclick: move |_| color.set(c.to_string()) }
                        }
                    }
                }
                div { class: "flex justify-end gap-2 pt-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), {tr("Abbrechen")} }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", disabled: name().trim().is_empty(), {tr("Speichern")} }
                }
            }
        }
    }
}

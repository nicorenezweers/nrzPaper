//! Einstellungen der Organisation: Allgemein & Mitglieder, Tagging-Regeln, Eigenschaften, Webhooks.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::*;
use crate::state::{self, ME, OVERVIEW};
use crate::util;
use crate::Route;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;

fn org_tags(org: &str) -> Vec<Tag> {
    OVERVIEW.read().clone().filter(|(o, _)| o == org).map(|(_, v)| v.tags).unwrap_or_default()
}

// ------------------------------------------------------------------ Allgemein & Mitglieder

#[derive(Deserialize, Clone, PartialEq, Default)]
struct Members {
    members: Vec<Member>,
}

#[component]
pub fn Organization(org: String, role: String) -> Element {
    let nav = use_navigator();
    let current = ME.read().clone().and_then(|m| m.organizations.into_iter().find(|o| o.id == org)).unwrap_or_default();
    let mut name = use_signal(|| current.name.clone());
    let o = org.clone();
    let members = use_resource(use_reactive!(|o| async move { api::get::<Members>(&format!("/organizations/{o}/members")).await.map(|m| m.members).unwrap_or_default() }));
    let manager = matches!(role.as_str(), "owner" | "admin");
    let (org_r, org_d) = (org.clone(), org.clone());
    let org_name = current.name.clone();
    rsx! {
        Section { title: "Allgemein", description: "Name der Organisation und Eingangsordner.",
            if manager {
                form { class: "flex max-w-lg gap-2",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let o = org_r.clone();
                        spawn(async move {
                            match api::put::<serde_json::Value, _>(&format!("/organizations/{o}"), &json!({ "name": name() })).await {
                                Ok(_) => { toast("Umbenannt."); state::load_me().await; }
                                Err(e) => toast_error(e),
                            }
                        });
                    },
                    input { class: INPUT, required: true, maxlength: 100, value: "{name}", oninput: move |e| name.set(e.value()) }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", "Umbenennen" }
                }
            } else {
                p { class: "font-medium", "{current.name}" }
            }
            p { class: "mt-4 text-sm text-muted-foreground", "Eingangsordner: " code { class: "rounded bg-muted px-1.5 py-0.5 font-mono text-xs", "ingestion/{org}/" } " – Dateien dort werden automatisch übernommen." }
        }
        Section { title: "Mitglieder",
            div { class: "divide-y rounded-lg border",
                for m in members.read().clone().unwrap_or_default() {
                    div { key: "{m.id}", class: "flex items-center justify-between px-4 py-2.5 text-sm",
                        span { class: "flex items-center gap-2.5",
                            span { class: "grid size-7 place-items-center rounded-full bg-secondary text-xs font-semibold", "{m.username.chars().next().unwrap_or('?').to_uppercase()}" }
                            "{m.username}"
                        }
                        span { class: "text-muted-foreground", "{role_label(&m.role)} · seit {util::date(m.since)}" }
                    }
                }
            }
        }
        if role == "owner" {
            section { class: "rounded-xl border border-destructive/40 p-5",
                h2 { class: "font-semibold", "Organisation löschen" }
                p { class: "mt-1 text-sm text-muted-foreground", "Die Organisation verschwindet für alle Mitglieder – samt Dokumenten, Tags, Ordnern und Webhooks." }
                button { class: "{btn(BTN_DANGER)} mt-4",
                    onclick: move |_| {
                        let (o, n) = (org_d.clone(), org_name.clone());
                        spawn(async move {
                            if confirm(Ask::new(format!("„{n}“ wirklich löschen?"), "Alle Dokumente dieser Organisation sind danach weg.").confirm("Organisation löschen").destructive()).await {
                                match api::delete(&format!("/organizations/{o}")).await {
                                    Ok(()) => { toast("Organisation gelöscht."); state::load_me().await; nav.replace(Route::Home {}); }
                                    Err(e) => toast_error(e),
                                }
                            }
                        });
                    },
                    "Organisation löschen"
                }
            }
        }
    }
}

// ------------------------------------------------------------------ Tagging-Regeln

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct Rules {
    tagging_rules: Vec<TaggingRule>,
}

#[component]
pub fn TaggingRules(org: String) -> Element {
    let mut version = use_signal(|| 0u64);
    let o = org.clone();
    let rules = use_resource(use_reactive!(|o| async move {
        let _ = version();
        api::get::<Rules>(&format!("/organizations/{o}/tagging-rules")).await.map(|r| r.tagging_rules).unwrap_or_default()
    }));
    let mut open = use_signal(|| false);
    let mut editing = use_signal(|| None::<TaggingRule>);
    let tags = org_tags(&org);
    rsx! {
        Section { title: "Tagging-Regeln",
            description: "Vergeben Tags automatisch, sobald der Text eines Dokuments erkannt ist – z. B. „enthält Swisscom“ → Rechnung, Telefon.",
            actions: rsx! { button { class: btn(BTN_PRIMARY), onclick: move |_| { editing.set(None); open.set(true); }, icons::Plus { class: "size-4" } "Neue Regel" } },
            match rules.read().clone() {
                None => rsx! { Loading {} },
                Some(list) if list.is_empty() => rsx! { p { class: "text-sm text-muted-foreground", "Noch keine Regeln." } },
                Some(list) => rsx! {
                    div { class: "grid gap-3",
                        for r in list {
                            div { key: "{r.id}", class: "rounded-lg border p-4",
                                div { class: "flex flex-wrap items-start gap-3",
                                    div { class: "min-w-0 flex-1",
                                        p { class: "flex items-center gap-2 font-medium",
                                            "{r.name}"
                                            if !r.enabled { span { class: "rounded-full border px-2 py-0.5 text-[11px] font-normal text-muted-foreground", "aus" } }
                                        }
                                        if !r.description.is_empty() { p { class: "text-sm text-muted-foreground", "{r.description}" } }
                                        p { class: "mt-2 text-sm text-muted-foreground",
                                            if r.match_mode == "any" { "Wenn eine trifft: " } else { "Wenn alle treffen: " }
                                            {r.conditions.iter().map(|c| format!("{} {} „{}“", if c.field == "name" { "Name" } else { "Text" }, OPERATORS.iter().find(|(k, _)| *k == c.operator).map(|(_, l)| *l).unwrap_or(""), c.value)).collect::<Vec<_>>().join(" · ")}
                                        }
                                        div { class: "mt-2 flex flex-wrap items-center gap-1.5 text-sm",
                                            icons::ArrowRight { class: "size-4 text-muted-foreground" }
                                            for t in tags.iter().filter(|t| r.tag_ids.contains(&t.id)).cloned() { TagBadge { tag: t, small: true } }
                                        }
                                    }
                                    div { class: "flex gap-1",
                                        button { class: "{btn(BTN_OUTLINE)} {BTN_SM}", title: "Auf alle vorhandenen Dokumente anwenden",
                                            onclick: { let (o, r) = (org.clone(), r.clone()); move |_| {
                                                let (o, r) = (o.clone(), r.clone());
                                                spawn(async move {
                                                    if confirm(Ask::new(format!("„{}“ auf alle Dokumente anwenden?", r.name), "Passende Dokumente bekommen die Tags der Regel. Vorhandene Tags bleiben.").confirm("Anwenden")).await {
                                                        match api::action::<serde_json::Value>(&format!("/organizations/{o}/tagging-rules/{}/apply", r.id)).await {
                                                            Ok(v) => { toast(format!("{} Dokument(e) passen, {} Tag(s) gesetzt.", v["matched"], v["tagsAdded"])); state::refresh(); }
                                                            Err(e) => toast_error(e),
                                                        }
                                                    }
                                                });
                                            } },
                                            icons::Wand { class: "size-4" } "Anwenden"
                                        }
                                        button { class: "{btn(BTN_GHOST)} {BTN_ICON}", title: "Bearbeiten", onclick: { let r = r.clone(); move |_| { editing.set(Some(r.clone())); open.set(true); } }, icons::Pencil { class: "size-4" } }
                                        button { class: "{btn(BTN_GHOST)} {BTN_ICON} text-muted-foreground hover:text-destructive", title: "Löschen",
                                            onclick: { let (o, r) = (org.clone(), r.clone()); move |_| {
                                                let (o, r) = (o.clone(), r.clone());
                                                spawn(async move {
                                                    if confirm(Ask::new(format!("Regel „{}“ löschen?", r.name), "Bereits vergebene Tags bleiben.").confirm("Löschen").destructive()).await {
                                                        match api::delete(&format!("/organizations/{o}/tagging-rules/{}", r.id)).await {
                                                            Ok(()) => { toast("Regel gelöscht."); version += 1; }
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
                    }
                },
            }
        }
        RuleDialog { open, org: org.clone(), rule: editing(), tags: tags.clone(), on_saved: move |_| version += 1 }
    }
}

#[component]
fn RuleDialog(open: Signal<bool>, org: String, rule: Option<TaggingRule>, tags: Vec<Tag>, on_saved: EventHandler<()>) -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut enabled = use_signal(|| true);
    let mut mode = use_signal(|| "all".to_string());
    let mut conditions = use_signal(Vec::<Condition>::new);
    let mut chosen = use_signal(Vec::<String>::new);
    let init = rule.clone();
    use_effect(use_reactive!(|init| {
        if open() {
            let r = init.clone().unwrap_or_default();
            name.set(r.name.clone());
            description.set(r.description.clone());
            enabled.set(init.is_none() || r.enabled);
            mode.set(if r.match_mode.is_empty() { "all".into() } else { r.match_mode.clone() });
            conditions.set(if r.conditions.is_empty() { vec![Condition { field: "content".into(), operator: "contains".into(), ..Default::default() }] } else { r.conditions.clone() });
            chosen.set(r.tag_ids.clone());
        }
    }));
    let id = rule.as_ref().map(|r| r.id.clone());
    let title = if rule.is_some() { "Regel bearbeiten" } else { "Neue Tagging-Regel" };
    rsx! {
        Modal { open, title, class: "max-w-2xl",
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    let (o, id) = (org.clone(), id.clone());
                    spawn(async move {
                        let body = json!({ "name": name(), "description": description(), "enabled": enabled(), "matchMode": mode(), "conditions": conditions(), "tagIds": chosen() });
                        let res: api::ApiResult<serde_json::Value> = match &id {
                            Some(id) => api::put(&format!("/organizations/{o}/tagging-rules/{id}"), &body).await,
                            None => api::post(&format!("/organizations/{o}/tagging-rules"), &body).await,
                        };
                        match res {
                            Ok(_) => { toast("Regel gespeichert."); open.set(false); on_saved.call(()); }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                div { class: "grid gap-4 sm:grid-cols-2",
                    Field { label: "Name", input { class: INPUT, required: true, placeholder: "z. B. Swisscom-Rechnungen", value: "{name}", oninput: move |e| name.set(e.value()) } }
                    Field { label: "Beschreibung", input { class: INPUT, value: "{description}", oninput: move |e| description.set(e.value()) } }
                }
                div { class: "flex flex-wrap items-center justify-between gap-3",
                    div { class: "flex rounded-md border p-0.5 text-sm",
                        button { r#type: "button", class: if mode() == "all" { "rounded-sm bg-accent px-3 py-1" } else { "rounded-sm px-3 py-1 text-muted-foreground" }, onclick: move |_| mode.set("all".into()), "Alle Bedingungen" }
                        button { r#type: "button", class: if mode() == "any" { "rounded-sm bg-accent px-3 py-1" } else { "rounded-sm px-3 py-1 text-muted-foreground" }, onclick: move |_| mode.set("any".into()), "Mindestens eine" }
                    }
                    Toggle { checked: enabled(), onchange: move |v| enabled.set(v), label: "Aktiv" }
                }
                div { class: "grid gap-2",
                    span { class: "text-sm font-medium", "Bedingungen" }
                    for (i , c) in conditions().into_iter().enumerate() {
                        div { key: "{i}", class: "grid grid-cols-[7rem_10rem_1fr_auto] items-center gap-2",
                            select { class: SELECT, onchange: move |e| conditions.write()[i].field = e.value(),
                                option { value: "content", selected: c.field == "content", "Text" }
                                option { value: "name", selected: c.field == "name", "Name" }
                            }
                            select { class: SELECT, onchange: move |e| conditions.write()[i].operator = e.value(),
                                for (k , l) in OPERATORS.iter().copied() { option { value: k, selected: c.operator == k, "{l}" } }
                            }
                            input { class: INPUT, required: true, placeholder: if c.operator == "regex" { "z. B. CH\\d{2}" } else { "Wert" }, value: "{c.value}", oninput: move |e| conditions.write()[i].value = e.value() }
                            button { r#type: "button", class: "{btn(BTN_GHOST)} {BTN_ICON}", title: "Entfernen", disabled: conditions().len() == 1, onclick: move |_| { conditions.write().remove(i); }, icons::X { class: "size-4" } }
                        }
                    }
                    button { r#type: "button", class: "{btn(BTN_GHOST)} {BTN_SM} justify-self-start",
                        onclick: move |_| conditions.write().push(Condition { field: "content".into(), operator: "contains".into(), ..Default::default() }),
                        icons::Plus { class: "size-4" } "Bedingung"
                    }
                }
                div { class: "grid gap-2",
                    span { class: "text-sm font-medium", "Dann diese Tags vergeben" }
                    if tags.is_empty() { p { class: "text-sm text-muted-foreground", "Zuerst unter Tags welche anlegen." } }
                    div { class: "flex flex-wrap gap-1.5",
                        for t in tags.iter().cloned() {
                            TagToggle { tag: t.clone(), on: chosen().contains(&t.id),
                                onclick: move |_| { let mut c = chosen.write(); if let Some(i) = c.iter().position(|x| *x == t.id) { c.remove(i); } else { c.push(t.id.clone()); } }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), "Abbrechen" }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", disabled: chosen().is_empty() || name().trim().is_empty(), "Speichern" }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ Eigenschaften

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct Defs {
    custom_properties: Vec<PropertyDefinition>,
}

#[component]
pub fn Properties(org: String) -> Element {
    let mut version = use_signal(|| 0u64);
    let o = org.clone();
    let defs = use_resource(use_reactive!(|o| async move {
        let _ = version();
        api::get::<Defs>(&format!("/organizations/{o}/custom-properties")).await.map(|d| d.custom_properties).unwrap_or_default()
    }));
    let mut open = use_signal(|| false);
    let mut editing = use_signal(|| None::<PropertyDefinition>);
    let o2 = org.clone();
    let reorder = use_callback(move |ids: Vec<String>| {
        let o = o2.clone();
        spawn(async move {
            match api::put::<serde_json::Value, _>(&format!("/organizations/{o}/custom-properties/order"), &json!({ "ids": ids })).await {
                Ok(_) => version += 1,
                Err(e) => toast_error(e),
            }
        });
    });
    rsx! {
        Section { title: "Eigenschaften",
            description: "Eigene Felder für Dokumente: Text, Zahl, Datum, Ja/Nein, Auswahl, Mehrfachauswahl, Benutzer oder Verweis auf ein Dokument. Die KI füllt Absender, Datum, Betrag & Co. beim Scannen aus.",
            actions: rsx! { button { class: btn(BTN_PRIMARY), onclick: move |_| { editing.set(None); open.set(true); }, icons::Plus { class: "size-4" } "Neue Eigenschaft" } },
            match defs.read().clone() {
                None => rsx! { Loading {} },
                Some(list) if list.is_empty() => rsx! { p { class: "text-sm text-muted-foreground", "Noch keine Eigenschaften." } },
                Some(list) => {
                    let ids: Vec<String> = list.iter().map(|d| d.id.clone()).collect();
                    rsx! {
                        div { class: "divide-y rounded-lg border",
                            for (i , d) in list.iter().cloned().enumerate() {
                                div { key: "{d.id}", class: "flex flex-wrap items-center gap-3 px-4 py-3",
                                    div { class: "flex flex-col",
                                        button { class: "text-muted-foreground hover:text-foreground disabled:opacity-30", disabled: i == 0, title: "Nach oben",
                                            onclick: { let mut ids = ids.clone(); move |_| { ids.swap(i, i - 1); reorder.call(ids.clone()); } },
                                            icons::ChevronUp { class: "size-4" }
                                        }
                                        button { class: "text-muted-foreground hover:text-foreground disabled:opacity-30", disabled: i + 1 == ids.len(), title: "Nach unten",
                                            onclick: { let mut ids = ids.clone(); move |_| { ids.swap(i, i + 1); reorder.call(ids.clone()); } },
                                            icons::ChevronDown { class: "size-4" }
                                        }
                                    }
                                    div { class: "min-w-0 flex-1",
                                        p { class: "font-medium", "{d.name} " span { class: "ml-1 rounded-full border px-2 py-0.5 text-[11px] font-normal text-muted-foreground", "{property_type_label(&d.kind)}" } }
                                        if !d.description.is_empty() { p { class: "text-sm text-muted-foreground", "{d.description}" } }
                                        if !d.options.is_empty() { p { class: "mt-1 text-xs text-muted-foreground", "{d.options.iter().map(|o| o.name.as_str()).collect::<Vec<_>>().join(\" · \")}" } }
                                    }
                                    button { class: "{btn(BTN_GHOST)} {BTN_ICON}", title: "Bearbeiten", onclick: { let d = d.clone(); move |_| { editing.set(Some(d.clone())); open.set(true); } }, icons::Pencil { class: "size-4" } }
                                    button { class: "{btn(BTN_GHOST)} {BTN_ICON} text-muted-foreground hover:text-destructive", title: "Löschen",
                                        onclick: { let (o, d) = (org.clone(), d.clone()); move |_| {
                                            let (o, d) = (o.clone(), d.clone());
                                            spawn(async move {
                                                if confirm(Ask::new(format!("Eigenschaft „{}“ löschen?", d.name), "Die Werte an allen Dokumenten gehen verloren.").confirm("Löschen").destructive()).await {
                                                    match api::delete(&format!("/organizations/{o}/custom-properties/{}", d.id)).await {
                                                        Ok(()) => { toast("Eigenschaft gelöscht."); version += 1; }
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
                }
            }
        }
        PropertyDialog { open, org: org.clone(), def: editing(), on_saved: move |_| version += 1 }
    }
}

#[component]
fn PropertyDialog(open: Signal<bool>, org: String, def: Option<PropertyDefinition>, on_saved: EventHandler<()>) -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut kind = use_signal(|| "text".to_string());
    let mut options = use_signal(Vec::<String>::new);
    let mut new_option = use_signal(String::new);
    let init = def.clone();
    use_effect(use_reactive!(|init| {
        if open() {
            let d = init.clone().unwrap_or_default();
            name.set(d.name.clone());
            description.set(d.description.clone());
            kind.set(if d.kind.is_empty() { "text".into() } else { d.kind.clone() });
            options.set(d.options.iter().map(|o| o.name.clone()).collect());
        }
    }));
    let id = def.as_ref().map(|d| d.id.clone());
    let editing = def.is_some();
    let has_options = matches!(kind().as_str(), "select" | "multi_select");
    rsx! {
        Modal { open, title: if editing { "Eigenschaft bearbeiten" } else { "Neue Eigenschaft" },
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    let (o, id) = (org.clone(), id.clone());
                    spawn(async move {
                        let body = json!({ "name": name(), "description": description(), "type": kind(), "options": options() });
                        let res: api::ApiResult<serde_json::Value> = match &id {
                            Some(id) => api::put(&format!("/organizations/{o}/custom-properties/{id}"), &body).await,
                            None => api::post(&format!("/organizations/{o}/custom-properties"), &body).await,
                        };
                        match res {
                            Ok(_) => { toast("Eigenschaft gespeichert."); open.set(false); on_saved.call(()); }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                Field { label: "Name", input { class: INPUT, required: true, maxlength: 64, placeholder: "z. B. Betrag", value: "{name}", oninput: move |e| name.set(e.value()) } }
                Field { label: "Typ", hint: if editing { "Der Typ lässt sich nachträglich nicht ändern." } else { "" },
                    select { class: SELECT, disabled: editing, onchange: move |e| kind.set(e.value()),
                        for (k , l) in PROPERTY_TYPES.iter().copied() { option { value: k, selected: kind() == k, "{l}" } }
                    }
                }
                Field { label: "Beschreibung", input { class: INPUT, value: "{description}", oninput: move |e| description.set(e.value()) } }
                if has_options {
                    div { class: "grid gap-2",
                        span { class: "text-sm font-medium", "Möglichkeiten" }
                        div { class: "flex flex-wrap gap-1.5",
                            for (i , o) in options().into_iter().enumerate() {
                                span { class: "inline-flex items-center gap-1 rounded-full border bg-secondary/60 px-2.5 py-0.5 text-xs",
                                    "{o}"
                                    button { r#type: "button", class: "text-muted-foreground hover:text-foreground", onclick: move |_| { options.write().remove(i); }, icons::X { class: "size-3" } }
                                }
                            }
                        }
                        div { class: "flex gap-2",
                            input { class: INPUT, placeholder: "Neue Möglichkeit + Enter", value: "{new_option}", oninput: move |e| new_option.set(e.value()),
                                onkeydown: move |e| if e.key() == Key::Enter {
                                    e.prevent_default();
                                    let v = new_option().trim().to_string();
                                    if !v.is_empty() && !options().iter().any(|o| o.eq_ignore_ascii_case(&v)) { options.write().push(v); }
                                    new_option.set(String::new());
                                }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), "Abbrechen" }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", disabled: name().trim().is_empty() || (has_options && options().is_empty()), "Speichern" }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ Webhooks

#[derive(Deserialize, Clone, PartialEq, Default)]
struct Hooks {
    webhooks: Vec<Webhook>,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
struct Deliveries {
    deliveries: Vec<Delivery>,
}

#[component]
pub fn Webhooks(org: String) -> Element {
    let mut version = use_signal(|| 0u64);
    let o = org.clone();
    let hooks = use_resource(use_reactive!(|o| async move {
        let _ = version();
        api::get::<Hooks>(&format!("/organizations/{o}/webhooks")).await.map(|h| h.webhooks).unwrap_or_default()
    }));
    let mut open = use_signal(|| false);
    let mut shown = use_signal(|| false);
    let mut secret = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut url = use_signal(String::new);
    let mut own_secret = use_signal(String::new);
    let mut events = use_signal(|| WEBHOOK_EVENTS.iter().map(|(e, _)| e.to_string()).collect::<Vec<_>>());
    let org_c = org.clone();
    rsx! {
        Section { title: "Webhooks",
            description: "Bei Ereignissen schickt nrzPaper ein signiertes JSON (Standard Webhooks: webhook-id, webhook-timestamp, webhook-signature) an die URL. Bis zu drei Versuche.",
            actions: rsx! { button { class: btn(BTN_PRIMARY), onclick: move |_| { name.set(String::new()); url.set(String::new()); own_secret.set(String::new()); open.set(true); }, icons::Plus { class: "size-4" } "Neuer Webhook" } },
            match hooks.read().clone() {
                None => rsx! { Loading {} },
                Some(list) if list.is_empty() => rsx! { p { class: "text-sm text-muted-foreground", "Noch keine Webhooks." } },
                Some(list) => rsx! {
                    div { class: "grid gap-3",
                        for h in list { HookCard { key: "{h.id}", org: org.clone(), hook: h, on_change: move |_| version += 1 } }
                    }
                },
            }
        }
        Modal { open, title: "Neuer Webhook", class: "max-w-2xl",
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    let o = org_c.clone();
                    spawn(async move {
                        let s = own_secret();
                        let body = json!({ "name": name(), "url": url(), "secret": if s.trim().is_empty() { None } else { Some(s) }, "events": events() });
                        match api::post::<serde_json::Value, _>(&format!("/organizations/{o}/webhooks"), &body).await {
                            Ok(v) => { secret.set(v["secret"].as_str().unwrap_or_default().to_string()); open.set(false); shown.set(true); version += 1; }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                div { class: "grid gap-4 sm:grid-cols-[1fr_2fr]",
                    Field { label: "Name", input { class: INPUT, required: true, placeholder: "z. B. Buchhaltung", value: "{name}", oninput: move |e| name.set(e.value()) } }
                    Field { label: "URL", input { class: INPUT, r#type: "url", required: true, placeholder: "https://…", value: "{url}", oninput: move |e| url.set(e.value()) } }
                }
                Field { label: "Geheimnis", hint: "Leer lassen = wird erzeugt und einmal angezeigt",
                    input { class: "{INPUT} font-mono", autocomplete: "off", value: "{own_secret}", oninput: move |e| own_secret.set(e.value()) }
                }
                div { class: "grid gap-2",
                    span { class: "text-sm font-medium", "Ereignisse" }
                    div { class: "grid gap-2 sm:grid-cols-2",
                        for (e , l) in WEBHOOK_EVENTS.iter().copied() {
                            Checkbox { label: l, hint: e.to_string(), checked: events().iter().any(|x| x == e),
                                onchange: move |on: bool| { let mut v = events(); v.retain(|x| x != e); if on { v.push(e.to_string()); } events.set(v); }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), "Abbrechen" }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", disabled: events().is_empty(), "Webhook anlegen" }
                }
            }
        }
        Modal { open: shown, title: "Geheimnis des Webhooks", description: "Jetzt kopieren – es wird nicht wieder angezeigt. Damit prüft der Empfänger die Signatur.", dismissible: false,
            div { class: "flex gap-2",
                input { class: "{INPUT} font-mono", readonly: true, value: "{secret}" }
                button { class: btn(BTN_PRIMARY), onclick: move |_| { let s = secret(); spawn(async move { if util::copy(&s).await { toast("Kopiert."); } }); }, icons::Copy { class: "size-4" } "Kopieren" }
            }
            div { class: "mt-5 flex justify-end", button { class: btn(BTN_OUTLINE), onclick: move |_| { shown.set(false); secret.set(String::new()); }, "Fertig" } }
        }
    }
}

#[component]
fn HookCard(org: String, hook: Webhook, on_change: EventHandler<()>) -> Element {
    let mut show = use_signal(|| false);
    let (o, id) = (org.clone(), hook.id.clone());
    let deliveries = use_resource(use_reactive!(|o, id| async move {
        if !show() {
            return Vec::new();
        }
        api::get::<Deliveries>(&format!("/organizations/{o}/webhooks/{id}/deliveries")).await.map(|d| d.deliveries).unwrap_or_default()
    }));
    let (org_t, h_t) = (org.clone(), hook.clone());
    let (org_d, h_d) = (org.clone(), hook.clone());
    rsx! {
        div { class: "rounded-lg border p-4",
            div { class: "flex flex-wrap items-center gap-3",
                span { class: if hook.enabled { "size-2 rounded-full bg-success" } else { "size-2 rounded-full bg-muted-foreground/40" } }
                div { class: "min-w-0 flex-1",
                    p { class: "font-medium", "{hook.name}" if !hook.enabled { span { class: "ml-2 rounded-full border px-2 py-0.5 text-[11px] font-normal text-muted-foreground", "pausiert" } } }
                    p { class: "truncate font-mono text-xs text-muted-foreground", "{hook.url}" }
                }
                Toggle { checked: hook.enabled,
                    onchange: move |on: bool| {
                        let (o, h) = (org_t.clone(), h_t.clone());
                        spawn(async move {
                            match api::patch::<serde_json::Value, _>(&format!("/organizations/{o}/webhooks/{}", h.id), &json!({ "enabled": on })).await {
                                Ok(_) => on_change.call(()),
                                Err(e) => toast_error(e),
                            }
                        });
                    }
                }
                button { class: "{btn(BTN_GHOST)} {BTN_ICON} text-muted-foreground hover:text-destructive", title: "Löschen",
                    onclick: move |_| {
                        let (o, h) = (org_d.clone(), h_d.clone());
                        spawn(async move {
                            if confirm(Ask::new(format!("Webhook „{}“ löschen?", h.name), "Es werden keine Ereignisse mehr an diese URL geschickt.").confirm("Löschen").destructive()).await {
                                match api::delete(&format!("/organizations/{o}/webhooks/{}", h.id)).await {
                                    Ok(()) => { toast("Webhook gelöscht."); on_change.call(()); }
                                    Err(e) => toast_error(e),
                                }
                            }
                        });
                    },
                    icons::Trash2 { class: "size-4" }
                }
            }
            div { class: "mt-2 flex flex-wrap gap-1",
                for e in hook.events.iter() {
                    span { class: "rounded-full border px-2 py-0.5 text-[11px] text-muted-foreground", "{WEBHOOK_EVENTS.iter().find(|(k, _)| k == e).map(|(_, l)| *l).unwrap_or(e)}" }
                }
            }
            button { class: "mt-3 text-xs text-muted-foreground hover:text-foreground", onclick: move |_| show.toggle(), if show() { "Zustellungen ausblenden" } else { "Letzte Zustellungen anzeigen" } }
            if show() {
                div { class: "mt-2 divide-y rounded-md border text-sm",
                    for d in deliveries.read().clone().unwrap_or_default() {
                        div { class: "flex flex-wrap items-center gap-3 px-3 py-1.5",
                            span { class: if d.error.is_none() { "w-10 font-mono text-xs" } else { "w-10 font-mono text-xs text-destructive" }, "{d.response_status.map(|s| s.to_string()).unwrap_or(\"–\".into())}" }
                            span { "{d.event}" }
                            span { class: "min-w-0 flex-1 truncate text-xs text-muted-foreground", "{d.error.clone().unwrap_or_default()}" }
                            span { class: "text-xs text-muted-foreground", "{util::datetime(d.created_at)}" }
                        }
                    }
                }
            }
        }
    }
}

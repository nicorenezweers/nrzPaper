//! Instanz-Einstellungen (nur Administratoren): KI über OpenRouter, Registrierung und Benutzer.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::*;
use crate::util;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Clone, PartialEq, Default)]
struct ModelInfo {
    id: String,
    name: String,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
struct Models {
    models: Vec<ModelInfo>,
}

fn source_label(s: &str) -> &'static str {
    match s {
        "settings" => "Eigener Schlüssel (hier hinterlegt)",
        "environment" => "Umgebungsvariable NRZPAPER_OPENROUTER_KEY",
        "builtin" => "Eingebauter Schlüssel",
        _ => "Kein Schlüssel",
    }
}

#[component]
pub fn Ai() -> Element {
    let mut current = use_signal(|| None::<AiSettings>);
    let mut disabled = use_signal(|| false);
    let mut key = use_signal(String::new);
    let mut model = use_signal(String::new);
    let mut fallbacks = use_signal(Vec::<String>::new);
    let mut new_fallback = use_signal(String::new);
    let mut models = use_signal(Vec::<ModelInfo>::new);
    let mut loading_models = use_signal(|| false);
    let mut testing = use_signal(|| false);
    let mut result = use_signal(|| None::<Result<String, String>>);

    let mut apply = move |s: AiSettings| {
        disabled.set(s.disabled);
        model.set(s.model.clone());
        fallbacks.set(s.fallbacks.clone());
        key.set(String::new());
        current.set(Some(s));
    };
    use_future(move || async move {
        match api::get::<AiSettings>("/settings/ai").await {
            Ok(s) => apply(s),
            Err(e) => toast_error(e),
        }
    });
    let save = move |clear_key: bool| {
        spawn(async move {
            let body = json!({ "disabled": disabled(), "key": key(), "clearKey": clear_key, "model": model(), "fallbacks": fallbacks() });
            match api::put::<AiSettings, _>("/settings/ai", &body).await {
                Ok(s) => { apply(s); toast("KI-Einstellungen gespeichert."); }
                Err(e) => toast_error(e),
            }
        });
    };
    let load_models = move |_| {
        loading_models.set(true);
        spawn(async move {
            match api::get::<Models>("/settings/ai/models").await {
                Ok(m) => models.set(m.models),
                Err(e) => toast_error(e),
            }
            loading_models.set(false);
        });
    };

    let Some(cur) = current() else { return rsx! { Loading {} } };
    let own_key = cur.key_source == "settings";
    rsx! {
        Section { title: "Status",
            div { class: "grid gap-3 sm:grid-cols-3",
                div { class: "rounded-lg border p-4",
                    p { class: "text-xs text-muted-foreground", "KI-Analyse" }
                    p { class: "mt-1 flex items-center gap-2 font-medium",
                        span { class: if cur.available { "size-2 rounded-full bg-success" } else { "size-2 rounded-full bg-destructive" } }
                        if cur.disabled { "Ausgeschaltet" } else if cur.available { "Bereit" } else { "Nicht verfügbar" }
                    }
                }
                div { class: "rounded-lg border p-4",
                    p { class: "text-xs text-muted-foreground", "Schlüssel" }
                    p { class: "mt-1 font-medium", "{source_label(&cur.key_source)}" }
                    if !cur.key_hint.is_empty() { p { class: "font-mono text-xs text-muted-foreground", "{cur.key_hint}" } }
                }
                div { class: "rounded-lg border p-4",
                    p { class: "text-xs text-muted-foreground", "Aktives Modell" }
                    p { class: "mt-1 break-all font-mono text-sm", "{cur.effective_model}" }
                    if !cur.effective_fallbacks.is_empty() { p { class: "text-xs text-muted-foreground", "+ {cur.effective_fallbacks.len()} Ausweichmodell(e)" } }
                }
            }
            div { class: "mt-4 flex flex-wrap items-center gap-3",
                button { class: btn(BTN_OUTLINE), disabled: testing() || !cur.available,
                    onclick: move |_| {
                        testing.set(true);
                        result.set(None);
                        spawn(async move {
                            match api::action::<serde_json::Value>("/ai/test").await {
                                Ok(v) => result.set(Some(Ok(format!("{} antwortet – Titel „{}“", v["model"].as_str().unwrap_or("?"), v["title"].as_str().unwrap_or(""))))),
                                Err(e) => result.set(Some(Err(e.message))),
                            }
                            testing.set(false);
                        });
                    },
                    if testing() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::FlaskConical { class: "size-4" } }
                    "Verbindung testen"
                }
                match result() {
                    Some(Ok(t)) => rsx! { span { class: "text-sm text-success", "{t}" } },
                    Some(Err(t)) => rsx! { span { class: "text-sm text-destructive", "{t}" } },
                    None => rsx! {},
                }
            }
        }
        Section { title: "OpenRouter",
            description: "Die KI liest gescannte Seiten und E-Mail-Anhänge, schlägt Titel, Tags und Eigenschaften vor. Ohne eigenen Schlüssel wird der eingebaute bzw. der aus der Umgebung verwendet.",
            form { class: "grid max-w-2xl gap-5",
                onsubmit: move |e| { e.prevent_default(); save(false); },
                Toggle { checked: !disabled(), onchange: move |on: bool| disabled.set(!on), label: "KI-Analyse verwenden" }
                Field { label: "API-Schlüssel", hint: if own_key { "Leer lassen = bisherigen behalten." } else { "Von openrouter.ai/keys – wird verschlüsselt gespeichert und nie wieder angezeigt." },
                    div { class: "flex gap-2",
                        input { class: "{INPUT} font-mono", r#type: "password", autocomplete: "off", placeholder: if own_key { cur.key_hint.clone() } else { "sk-or-v1-…".to_string() }, value: "{key}", oninput: move |e| key.set(e.value()) }
                        if own_key {
                            button { class: btn(BTN_OUTLINE), r#type: "button",
                                onclick: move |_| {
                                    spawn(async move {
                                        if confirm(Ask::new("Eigenen Schlüssel entfernen?", "Danach gilt wieder der Schlüssel aus der Umgebung bzw. der eingebaute.").confirm("Entfernen").destructive()).await {
                                            save(true);
                                        }
                                    });
                                },
                                "Entfernen"
                            }
                        }
                    }
                }
                Field { label: "Modell", hint: format!("Leer = Standard ({})", cur.default_model),
                    div { class: "flex gap-2",
                        input { class: "{INPUT} font-mono", list: "ai-models", placeholder: cur.default_model.clone(), value: "{model}", oninput: move |e| model.set(e.value()) }
                        button { class: btn(BTN_OUTLINE), r#type: "button", disabled: loading_models(), onclick: load_models, title: "Kostenlose Bild-Modelle von OpenRouter laden",
                            if loading_models() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::RefreshCw { class: "size-4" } }
                            "Modelle laden"
                        }
                    }
                    datalist { id: "ai-models",
                        for m in models() { option { value: "{m.id}", "{m.name}" } }
                    }
                    if !models().is_empty() {
                        div { class: "mt-2 flex max-h-48 flex-col overflow-y-auto rounded-md border text-sm",
                            for m in models() {
                                button { r#type: "button", class: if model() == m.id { "flex justify-between gap-3 bg-accent px-3 py-1.5 text-left" } else { "flex justify-between gap-3 px-3 py-1.5 text-left hover:bg-accent/50" },
                                    onclick: { let id = m.id.clone(); move |_| model.set(id.clone()) },
                                    span { "{m.name}" }
                                    span { class: "truncate font-mono text-xs text-muted-foreground", "{m.id}" }
                                }
                            }
                        }
                    }
                }
                div { class: "grid gap-2",
                    span { class: "text-sm font-medium", "Ausweichmodelle" }
                    p { class: "text-xs text-muted-foreground", "Werden der Reihe nach versucht, wenn das Modell ausgelastet ist. Leer = Standardliste ({cur.default_fallbacks.len()})." }
                    for (i , f) in fallbacks().into_iter().enumerate() {
                        div { key: "{i}-{f}", class: "flex items-center gap-2 rounded-md border px-3 py-1.5",
                            span { class: "w-5 text-xs text-muted-foreground", "{i + 1}." }
                            span { class: "min-w-0 flex-1 truncate font-mono text-sm", "{f}" }
                            button { r#type: "button", class: "text-muted-foreground hover:text-foreground disabled:opacity-30", disabled: i == 0, onclick: move |_| fallbacks.write().swap(i, i - 1), icons::ChevronUp { class: "size-4" } }
                            button { r#type: "button", class: "text-muted-foreground hover:text-destructive", onclick: move |_| { fallbacks.write().remove(i); }, icons::X { class: "size-4" } }
                        }
                    }
                    input { class: "{INPUT} font-mono", list: "ai-models", placeholder: "Modell-ID + Enter", value: "{new_fallback}", oninput: move |e| new_fallback.set(e.value()),
                        onkeydown: move |e| if e.key() == Key::Enter {
                            e.prevent_default();
                            let v = new_fallback().trim().to_string();
                            if !v.is_empty() && !fallbacks().contains(&v) { fallbacks.write().push(v); }
                            new_fallback.set(String::new());
                        }
                    }
                }
                div { button { class: btn(BTN_PRIMARY), r#type: "submit", "Speichern" } }
            }
        }
    }
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct InstanceInfo {
    registration_open: bool,
    trash_retention_days: i64,
    ocr_languages: String,
    version: String,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct UserRow {
    id: String,
    username: String,
    is_admin: bool,
    created_at: i64,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
struct Users {
    users: Vec<UserRow>,
}

#[component]
pub fn Instance() -> Element {
    let mut info = use_signal(|| None::<InstanceInfo>);
    use_future(move || async move {
        match api::get::<InstanceInfo>("/settings/instance").await {
            Ok(i) => info.set(Some(i)),
            Err(e) => toast_error(e),
        }
    });
    let users = use_resource(|| async { api::get::<Users>("/settings/users").await.map(|u| u.users).unwrap_or_default() });
    let Some(i) = info() else { return rsx! { Loading {} } };
    rsx! {
        Section { title: "Registrierung",
            Toggle { checked: i.registration_open, label: "Neue Benutzer dürfen sich selbst registrieren",
                onchange: move |on: bool| {
                    spawn(async move {
                        match api::put::<InstanceInfo, _>("/settings/instance", &json!({ "registrationOpen": on })).await {
                            Ok(i) => { info.set(Some(i)); toast(if on { "Registrierung geöffnet." } else { "Registrierung geschlossen." }); }
                            Err(e) => toast_error(e),
                        }
                    });
                }
            }
            dl { class: "mt-5 grid max-w-md grid-cols-[auto_1fr] gap-x-6 gap-y-1.5 text-sm",
                dt { class: "text-muted-foreground", "Version" } dd { "{i.version}" }
                dt { class: "text-muted-foreground", "Papierkorb" } dd { "{i.trash_retention_days} Tage" }
                dt { class: "text-muted-foreground", "OCR-Sprachen" } dd { class: "font-mono", "{i.ocr_languages}" }
            }
        }
        Section { title: "Benutzer",
            div { class: "divide-y rounded-lg border",
                for u in users.read().clone().unwrap_or_default() {
                    div { key: "{u.id}", class: "flex items-center justify-between px-4 py-2.5 text-sm",
                        span { class: "flex items-center gap-2.5",
                            span { class: "grid size-7 place-items-center rounded-full bg-secondary text-xs font-semibold", "{u.username.chars().next().unwrap_or('?').to_uppercase()}" }
                            "{u.username}"
                            if u.is_admin { span { class: "rounded-full border px-2 py-0.5 text-[11px] text-muted-foreground", "Admin" } }
                        }
                        span { class: "text-muted-foreground", "seit {util::date(u.created_at)}" }
                    }
                }
            }
        }
    }
}

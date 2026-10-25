//! Instanz-Einstellungen (nur Administratoren): KI-Anbieter und Modelle, Registrierung und Konten.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::*;
use crate::util;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;
use crate::i18n::{tr, trf};

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct ModelInfo {
    id: String,
    name: String,
    free: bool,
    vision: Option<bool>,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
struct Models {
    models: Vec<ModelInfo>,
}

fn source_label(s: &str) -> String {
    match s {
        "settings" => {tr("Hier hinterlegt")},
        "environment" => {tr("Aus der Umgebung (NRZPAPER_AI_KEY / NRZPAPER_OPENROUTER_KEY)")},
        "builtin" => {tr("Eingebauter Schlüssel")},
        _ => {tr("Kein Schlüssel")},
    }
}

/// Anbieter in Gruppen für die Auswahl.
fn group(id: &str) -> &'static str {
    match id {
        "ollama" | "lmstudio" | "vllm" => "Lokal (eigener Rechner)",
        "custom" | "azure" => "Eigene Adresse",
        _ => "Cloud",
    }
}

#[component]
pub fn Ai() -> Element {
    let mut current = use_signal(|| None::<AiSettings>);
    let mut disabled = use_signal(|| false);
    let mut provider = use_signal(String::new);
    let mut key = use_signal(String::new);
    let mut url = use_signal(String::new);
    let mut model = use_signal(String::new);
    let mut fallbacks = use_signal(Vec::<String>::new);
    let mut text_only = use_signal(|| false);
    let mut language = use_signal(|| "de".to_string());
    let mut new_fallback = use_signal(String::new);
    let mut models = use_signal(Vec::<ModelInfo>::new);
    let mut model_filter = use_signal(String::new);
    let mut only_free = use_signal(|| false);
    let mut loading_models = use_signal(|| false);
    let mut testing = use_signal(|| false);
    let mut result = use_signal(|| None::<Result<String, String>>);

    let mut apply = move |s: AiSettings| {
        disabled.set(s.disabled);
        provider.set(s.provider.clone());
        model.set(s.model.clone());
        fallbacks.set(s.fallbacks.clone());
        text_only.set(s.text_only);
        language.set(if s.language.is_empty() { "de".into() } else { s.language.clone() });
        url.set(s.providers.iter().find(|p| p.id == s.provider).map(|p| p.custom_url.clone()).unwrap_or_default());
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
            let body = json!({
                "disabled": disabled(), "provider": provider(), "key": key(), "clearKey": clear_key,
                "url": url(), "model": model(), "fallbacks": fallbacks(), "textOnly": text_only(), "language": language(),
            });
            match api::put::<AiSettings, _>("/settings/ai", &body).await {
                Ok(s) => { apply(s); toast(tr("KI-Einstellungen gespeichert.")); }
                Err(e) => toast_error(e),
            }
        });
    };
    let load_models = move |_| {
        loading_models.set(true);
        spawn(async move {
            // Zuerst speichern, damit Schlüssel und Adresse gelten
            let body = json!({
                "disabled": disabled(), "provider": provider(), "key": key(), "url": url(),
                "model": model(), "fallbacks": fallbacks(), "textOnly": text_only(),
            });
            match api::put::<AiSettings, _>("/settings/ai", &body).await {
                Ok(s) => apply(s),
                Err(e) => { toast_error(e); loading_models.set(false); return; }
            }
            match api::get::<Models>("/settings/ai/models").await {
                Ok(m) => {
                    if m.models.is_empty() { toast(tr("Der Anbieter meldet keine Modelle.")); }
                    models.set(m.models);
                }
                Err(e) => toast_error(e),
            }
            loading_models.set(false);
        });
    };

    let Some(cur) = current() else { return rsx! { Loading {} } };
    let p = cur.providers.iter().find(|x| x.id == provider()).cloned().unwrap_or_default();
    let switched = provider() != cur.provider;
    let own_key = p.has_key;
    let show_url = p.url.is_empty() || p.key_optional || !url().is_empty() || p.id == "azure";
    let filter = model_filter().to_lowercase();
    let shown: Vec<ModelInfo> = models()
        .into_iter()
        .filter(|m| !only_free() || m.free)
        .filter(|m| filter.is_empty() || m.id.to_lowercase().contains(&filter) || m.name.to_lowercase().contains(&filter))
        .collect();
    let groups = ["Cloud", "Lokal (eigener Rechner)", "Eigene Adresse"];
    rsx! {
        Section { title: tr("Status"),
            div { class: "grid gap-3 sm:grid-cols-3",
                div { class: "rounded-lg border p-4",
                    p { class: "text-xs text-muted-foreground", {tr("KI-Analyse")} }
                    p { class: "mt-1 flex items-center gap-2 font-medium",
                        span { class: if cur.available { "size-2 rounded-full bg-success" } else { "size-2 rounded-full bg-destructive" } }
                        if cur.disabled { {tr("Ausgeschaltet")} } else if cur.available { {tr("Bereit")} } else { {tr("Nicht bereit")} }
                    }
                    if let Some(m) = cur.missing.as_ref() { p { class: "mt-1 text-xs text-muted-foreground", "{m}" } }
                }
                div { class: "rounded-lg border p-4",
                    p { class: "text-xs text-muted-foreground", {tr("Anbieter und Schlüssel")} }
                    p { class: "mt-1 font-medium", "{cur.providers.iter().find(|x| x.id == cur.provider).map(|x| x.name.clone()).unwrap_or_default()}" }
                    p { class: "text-xs text-muted-foreground", "{source_label(&cur.key_source)}" }
                    if !cur.key_hint.is_empty() { p { class: "font-mono text-xs text-muted-foreground", "{cur.key_hint}" } }
                }
                div { class: "rounded-lg border p-4",
                    p { class: "text-xs text-muted-foreground", {tr("Aktives Modell")} }
                    p { class: "mt-1 break-all font-mono text-sm", if cur.effective_model.is_empty() { {tr("–")} } else { "{cur.effective_model}" } }
                    if !cur.effective_fallbacks.is_empty() { p { class: "text-xs text-muted-foreground", {trf("+ {0} Ausweichmodell(e)", &[&(cur.effective_fallbacks.len())])} } }
                    if cur.text_only { p { class: "text-xs text-muted-foreground", {tr("liest den erkannten Text")} } }
                }
            }
            div { class: "mt-4 flex flex-wrap items-center gap-3",
                button { class: btn(BTN_OUTLINE), disabled: testing() || !cur.available,
                    onclick: move |_| {
                        testing.set(true);
                        result.set(None);
                        spawn(async move {
                            match api::action::<serde_json::Value>("/ai/test").await {
                                Ok(v) => result.set(Some(Ok(trf("{0} antwortet – Titel „{1}“", &[&(v["model"].as_str().unwrap_or("?")), &(v["title"].as_str().unwrap_or(""))])))),
                                Err(e) => result.set(Some(Err(e.message))),
                            }
                            testing.set(false);
                        });
                    },
                    if testing() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::FlaskConical { class: "size-4" } }
                    {tr("Verbindung testen")}
                }
                match result() {
                    Some(Ok(t)) => rsx! { span { class: "text-sm text-success", "{t}" } },
                    Some(Err(t)) => rsx! { span { class: "text-sm text-destructive", "{t}" } },
                    None => rsx! {},
                }
            }
        }
        Section { title: tr("Anbieter und Modell"),
            description: tr("Die KI liest gescannte Seiten, PDFs und E-Mail-Anhänge und schlägt Titel, Tags und Eigenschaften vor. Jeder Anbieter mit OpenAI-kompatibler API funktioniert, dazu Anthropic. Lokale Server (Ollama, LM Studio, vLLM) brauchen keinen Schlüssel."),
            form { class: "grid max-w-2xl gap-5",
                onsubmit: move |e| { e.prevent_default(); save(false); },
                Toggle { checked: !disabled(), onchange: move |on: bool| disabled.set(!on), label: tr("KI-Analyse verwenden") }
                Field { label: tr("Anbieter"),
                    select { class: SELECT,
                        onchange: move |e| {
                            let id = e.value();
                            let cur = current().unwrap_or_default();
                            url.set(cur.providers.iter().find(|x| x.id == id).map(|x| x.custom_url.clone()).unwrap_or_default());
                            if id != cur.provider { model.set(String::new()); fallbacks.set(Vec::new()); } else { model.set(cur.model.clone()); fallbacks.set(cur.fallbacks.clone()); }
                            key.set(String::new());
                            models.set(Vec::new());
                            provider.set(id);
                        },
                        for g in groups {
                            optgroup { label: tr(g),
                                for x in cur.providers.iter().filter(|x| group(&x.id) == g) {
                                    option { value: "{x.id}", selected: x.id == provider(), "{x.name}" if x.has_key { " ✓" } }
                                }
                            }
                        }
                    }
                }
                if switched {
                    Callout { text: trf("Noch nicht gespeichert – erst mit „Speichern“ gilt {0}.", &[&(p.name)]) }
                }
                Field { label: if p.key_optional { tr("API-Schlüssel (optional)") } else { tr("API-Schlüssel") },
                    hint: if own_key { trf("Hinterlegt ({0}). Leer lassen = behalten.", &[&(p.key_hint)]) } else if p.id == "openrouter" { {tr("Leer lassen = eingebauter Schlüssel (kostenlose Modelle).")}.to_string() } else { {tr("Wird verschlüsselt gespeichert und nie wieder angezeigt.")}.to_string() },
                    div { class: "flex gap-2",
                        input { class: "{INPUT} font-mono", r#type: "password", autocomplete: "off", placeholder: if own_key { p.key_hint.clone() } else { String::new() }, value: "{key}", oninput: move |e| key.set(e.value()) }
                        if own_key {
                            button { class: btn(BTN_OUTLINE), r#type: "button",
                                onclick: move |_| {
                                    spawn(async move {
                                        if confirm(Ask::new(tr("Schlüssel entfernen?"), tr("Der hinterlegte Schlüssel dieses Anbieters wird gelöscht.")).confirm(tr("Entfernen")).destructive()).await {
                                            save(true);
                                        }
                                    });
                                },
                                {tr("Entfernen")}
                            }
                        }
                    }
                    if !p.keys_url.is_empty() {
                        a { class: "mt-1 inline-flex items-center gap-1 text-xs text-muted-foreground underline-offset-4 hover:text-foreground hover:underline", href: "{p.keys_url}", target: "_blank", rel: "noopener noreferrer",
                            {trf("Schlüssel bei {0} holen", &[&(p.name)])} icons::ExternalLink { class: "size-3" }
                        }
                    }
                }
                if show_url {
                    Field { label: tr("Adresse der API"),
                        hint: if p.id == "azure" { {tr("z. B. https://NAME.openai.azure.com/openai/v1 – Modell = Name der Bereitstellung")}.to_string() } else if p.url.is_empty() { {tr("Basis-Adresse bis …/v1 (ohne /chat/completions)")}.to_string() } else { trf("Leer = {0}", &[&(p.url)]) },
                        input { class: "{INPUT} font-mono", r#type: "url", placeholder: if p.url.is_empty() { "https://…/v1".to_string() } else { p.url.clone() }, value: "{url}", oninput: move |e| url.set(e.value()) }
                    }
                } else {
                    button { r#type: "button", class: "justify-self-start text-xs text-muted-foreground hover:text-foreground", onclick: move |_| url.set(p.url.clone()), {tr("Eigene Adresse (Proxy, Gateway) …")} }
                }
                Field { label: tr("Modell"), hint: if p.model.is_empty() { {tr("Modell-ID des Anbieters – „Modelle laden“ zeigt die verfügbaren.")}.to_string() } else { trf("Leer = {0}", &[&(p.model)]) },
                    div { class: "flex gap-2",
                        input { class: "{INPUT} font-mono", list: "ai-models", placeholder: p.model.clone(), value: "{model}", oninput: move |e| model.set(e.value()) }
                        button { class: btn(BTN_OUTLINE), r#type: "button", disabled: loading_models(), onclick: load_models, title: tr("Modelle beim Anbieter abfragen"),
                            if loading_models() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::RefreshCw { class: "size-4" } }
                            {tr("Modelle laden")}
                        }
                    }
                    datalist { id: "ai-models",
                        for m in models() { option { value: "{m.id}", "{m.name}" } }
                    }
                    if !models().is_empty() {
                        div { class: "mt-2 flex flex-wrap items-center gap-3",
                            input { class: "{INPUT} h-8 max-w-xs", placeholder: tr("Modelle filtern …"), value: "{model_filter}", oninput: move |e| model_filter.set(e.value()) }
                            if models().iter().any(|m| m.free) {
                                Checkbox { checked: only_free(), onchange: move |v| only_free.set(v), label: tr("nur kostenlose") }
                            }
                            span { class: "text-xs text-muted-foreground", {trf("{0} von {1}", &[&shown.len(), &models().len()])} }
                        }
                        div { class: "mt-2 flex max-h-64 flex-col overflow-y-auto rounded-md border text-sm",
                            for m in shown.into_iter().take(300) {
                                button { r#type: "button", class: if model() == m.id { "flex items-center justify-between gap-3 bg-accent px-3 py-1.5 text-left" } else { "flex items-center justify-between gap-3 px-3 py-1.5 text-left hover:bg-accent/50" },
                                    onclick: { let id = m.id.clone(); move |_| model.set(id.clone()) },
                                    span { class: "flex min-w-0 items-center gap-2",
                                        span { class: "truncate", "{m.name}" }
                                        if m.free { span { class: "rounded-full border px-1.5 text-[10px] text-muted-foreground", {tr("kostenlos")} } }
                                        if m.vision == Some(true) { span { class: "rounded-full border px-1.5 text-[10px] text-muted-foreground", {tr("Bilder")} } }
                                        if m.vision == Some(false) { span { class: "rounded-full border px-1.5 text-[10px] text-muted-foreground", {tr("nur Text")} } }
                                    }
                                    span { class: "truncate font-mono text-xs text-muted-foreground", "{m.id}" }
                                }
                            }
                        }
                    }
                }
                Field { label: tr("Sprache der Vorschläge"), hint: tr("Titel und Zusammenfassung, die die KI vorschlägt."),
                    select { class: SELECT, onchange: move |e| language.set(e.value()),
                        option { value: "auto", selected: language() == "auto", {tr("Wie das Dokument")} }
                        for (code , name) in crate::i18n::LANGUAGES.iter().copied() {
                            option { value: code, selected: language() == code, "{name}" }
                        }
                    }
                }
                Checkbox { checked: text_only(), onchange: move |v| text_only.set(v),
                    label: tr("Modell liest keine Bilder – erkannten Text schicken"),
                    hint: tr("Für reine Textmodelle (z. B. DeepSeek, viele lokale Modelle): Seiten und PDFs gehen erst durch die Texterkennung.") }
                if !p.pdf && !text_only() {
                    p { class: "-mt-2 text-xs text-muted-foreground", {trf("{0} nimmt keine PDFs an – importierte PDFs gehen als Bilder der ersten vier Seiten.", &[&(p.name)])} }
                }
                div { class: "grid gap-2",
                    span { class: "text-sm font-medium", {tr("Ausweichmodelle")} }
                    p { class: "text-xs text-muted-foreground",
                        {tr("Werden der Reihe nach versucht, wenn das Modell ausgelastet ist oder Unbrauchbares liefert.")}
                        if !cur.default_fallbacks.is_empty() && !switched { {trf(" Leer = Standardliste ({0}).", &[&(cur.default_fallbacks.len())])} }
                    }
                    for (i , f) in fallbacks().into_iter().enumerate() {
                        div { key: "{i}-{f}", class: "flex items-center gap-2 rounded-md border px-3 py-1.5",
                            span { class: "w-5 text-xs text-muted-foreground", "{i + 1}." }
                            span { class: "min-w-0 flex-1 truncate font-mono text-sm", "{f}" }
                            button { r#type: "button", class: "text-muted-foreground hover:text-foreground disabled:opacity-30", disabled: i == 0, onclick: move |_| fallbacks.write().swap(i, i - 1), icons::ChevronUp { class: "size-4" } }
                            button { r#type: "button", class: "text-muted-foreground hover:text-destructive", onclick: move |_| { fallbacks.write().remove(i); }, icons::X { class: "size-4" } }
                        }
                    }
                    input { class: "{INPUT} font-mono", list: "ai-models", placeholder: tr("Modell-ID + Enter"), value: "{new_fallback}", oninput: move |e| new_fallback.set(e.value()),
                        onkeydown: move |e| if e.key() == Key::Enter {
                            e.prevent_default();
                            let v = new_fallback().trim().to_string();
                            if !v.is_empty() && !fallbacks().contains(&v) { fallbacks.write().push(v); }
                            new_fallback.set(String::new());
                        }
                    }
                }
                div { button { class: btn(BTN_PRIMARY), r#type: "submit", {tr("Speichern")} } }
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
    let mut version = use_signal(|| 0u64);
    let users = use_resource(move || async move {
        let _ = version();
        api::get::<Users>("/settings/users").await.map(|u| u.users).unwrap_or_default()
    });
    let mut create_open = use_signal(|| false);
    let me = crate::state::ME.read().as_ref().map(|m| m.user.id.clone()).unwrap_or_default();
    let Some(i) = info() else { return rsx! { Loading {} } };
    rsx! {
        Section { title: tr("Registrierung"),
            Toggle { checked: i.registration_open, label: tr("Neue Benutzer dürfen sich selbst registrieren"),
                onchange: move |on: bool| {
                    spawn(async move {
                        match api::put::<InstanceInfo, _>("/settings/instance", &json!({ "registrationOpen": on })).await {
                            Ok(i) => { info.set(Some(i)); toast(if on { tr("Registrierung geöffnet.") } else { tr("Registrierung geschlossen.") }); }
                            Err(e) => toast_error(e),
                        }
                    });
                }
            }
            dl { class: "mt-5 grid max-w-md grid-cols-[auto_1fr] gap-x-6 gap-y-1.5 text-sm",
                dt { class: "text-muted-foreground", {tr("Version")} } dd { "{i.version}" }
                dt { class: "text-muted-foreground", {tr("Papierkorb")} } dd { {trf("{0} Tage", &[&(i.trash_retention_days)])} }
                dt { class: "text-muted-foreground", {tr("OCR-Sprachen")} } dd { class: "font-mono", "{i.ocr_languages}" }
            }
        }
        Section { title: tr("Benutzer"),
            description: tr("Konten dieser Instanz. Neue Konten lassen sich hier anlegen – auch bei geschlossener Registrierung – und danach in Organisationen einladen."),
            actions: rsx! { button { class: btn(BTN_PRIMARY), onclick: move |_| create_open.set(true), icons::UserPlus { class: "size-4" } {tr("Konto anlegen")} } },
            div { class: "divide-y rounded-lg border",
                for u in users.read().clone().unwrap_or_default() {
                    div { key: "{u.id}", class: "flex items-center gap-3 px-4 py-2.5 text-sm",
                        span { class: "grid size-7 place-items-center rounded-full bg-secondary text-xs font-semibold", "{u.username.chars().next().unwrap_or('?').to_uppercase()}" }
                        span { class: "flex min-w-0 flex-1 items-center gap-2",
                            span { class: "truncate", "{u.username}" }
                            if u.is_admin { span { class: "rounded-full border px-2 py-0.5 text-[11px] text-muted-foreground", {tr("Admin")} } }
                            if u.id == me { span { class: "text-muted-foreground", {tr("(du)")} } }
                        }
                        span { class: "hidden text-muted-foreground sm:inline", {trf("seit {0}", &[&util::date(u.created_at)])} }
                        if u.id != me {
                            button { class: "{btn(BTN_GHOST)} {BTN_ICON} text-muted-foreground hover:text-destructive", title: tr("Konto löschen"),
                                onclick: { let u = u.clone(); move |_| {
                                    let u = u.clone();
                                    spawn(async move {
                                        if confirm(Ask::new(trf("Konto „{0}“ löschen?", &[&(u.username)]), tr("Die Person kann sich nicht mehr anmelden. Organisationen, die nur ihr gehören, verschwinden mit; Dokumente in gemeinsamen Organisationen bleiben.")).confirm(tr("Konto löschen")).destructive()).await {
                                            match api::delete(&format!("/settings/users/{}", u.id)).await {
                                                Ok(()) => { toast(tr("Konto gelöscht.")); version += 1; }
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
        CreateUser { open: create_open, on_done: move |_| version += 1 }
    }
}

#[component]
fn CreateUser(open: Signal<bool>, on_done: EventHandler<()>) -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut admin = use_signal(|| false);
    use_effect(move || {
        if open() {
            username.set(String::new());
            password.set(String::new());
            admin.set(false);
        }
    });
    rsx! {
        Modal { open, title: tr("Konto anlegen"), description: tr("Gib der Person Benutzername und Passwort weiter; sie kann das Passwort danach unter Profil & Passwort ändern."),
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    spawn(async move {
                        let body = json!({ "username": username().trim(), "password": password(), "isAdmin": admin() });
                        match api::post::<serde_json::Value, _>("/settings/users", &body).await {
                            Ok(_) => { toast(trf("Konto „{0}“ angelegt.", &[&(username().trim())])); open.set(false); on_done.call(()); }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                Field { label: tr("Benutzername"),
                    input { class: INPUT, required: true, autocomplete: "off", value: "{username}", oninput: move |e| username.set(e.value()),
                    }
                }
                Field { label: tr("Passwort"), hint: tr("Mindestens 8 Zeichen"),
                    input { class: INPUT, r#type: "password", required: true, minlength: 8, autocomplete: "new-password", value: "{password}", oninput: move |e| password.set(e.value()) }
                }
                Checkbox { checked: admin(), onchange: move |v| admin.set(v), label: tr("Administrator der Instanz"), hint: tr("Darf KI, Registrierung und Konten verwalten.") }
                div { class: "flex justify-end gap-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), {tr("Abbrechen")} }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", disabled: username().trim().is_empty() || password().len() < 8, {tr("Anlegen")} }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ Drucker

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct PrinterRow {
    id: String,
    name: String,
    url: String,
    make: String,
    formats: Vec<String>,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
struct PrinterRows {
    printers: Vec<PrinterRow>,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct FoundPrinter {
    url: String,
    name: String,
    make: String,
    formats: Vec<String>,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
struct Discovered {
    printers: Vec<FoundPrinter>,
    searched: Vec<String>,
}

fn formats_label(f: &[String]) -> String {
    let mut out = Vec::new();
    if f.iter().any(|x| x == "application/pdf") {
        out.push("PDF");
    }
    if f.iter().any(|x| x == "image/jpeg") {
        out.push("JPEG");
    }
    if f.iter().any(|x| x == "image/urf") {
        out.push("AirPrint");
    }
    if f.iter().any(|x| x == "image/pwg-raster") {
        out.push("IPP Everywhere");
    }
    out.join(" · ")
}

#[component]
pub fn PrintersSettings() -> Element {
    let mut version = use_signal(|| 0u64);
    let list = use_resource(move || async move {
        let _ = version();
        api::get::<PrinterRows>("/printers").await.map(|p| p.printers).unwrap_or_default()
    });
    let mut url = use_signal(String::new);
    let mut subnets = use_signal(String::new);
    let mut searching = use_signal(|| false);
    let mut found = use_signal(|| None::<Discovered>);
    let mut adding = use_signal(|| false);
    let mut rename_open = use_signal(|| false);
    let mut renaming = use_signal(|| None::<PrinterRow>);
    let mut new_name = use_signal(String::new);
    let add = move |u: String| {
        spawn(async move {
            adding.set(true);
            match api::post::<serde_json::Value, _>("/settings/printers", &json!({ "url": u })).await {
                Ok(v) => { toast(trf("Drucker „{0}“ eingerichtet.", &[&v["printer"]["name"].as_str().unwrap_or("")])); url.set(String::new()); version += 1; }
                Err(e) => toast_error(e),
            }
            adding.set(false);
        });
    };
    let rows = list.read().clone().unwrap_or_default();
    rsx! {
        Section { title: tr("Drucker"),
            description: tr("Netzwerkdrucker mit IPP (AirPrint, IPP Everywhere, Mopria) – praktisch jeder aktuelle Drucker von HP, Canon, Epson, Brother & Co., auch Drucker an einem CUPS-Server. Alle Mitglieder können darauf Dokumente drucken."),
            if rows.is_empty() {
                p { class: "text-sm text-muted-foreground", {tr("Noch kein Drucker eingerichtet.")} }
            }
            div { class: "grid gap-2",
                for p in rows.iter().cloned() {
                    div { key: "{p.id}", class: "flex flex-wrap items-center gap-3 rounded-lg border px-4 py-3",
                        icons::Printer { class: "size-5 text-muted-foreground" }
                        div { class: "min-w-0 flex-1",
                            p { class: "font-medium", "{p.name}" }
                            p { class: "truncate font-mono text-xs text-muted-foreground", "{p.url}" }
                            p { class: "text-xs text-muted-foreground", "{p.make} · {formats_label(&p.formats)}" }
                        }
                        button { class: "{btn(BTN_GHOST)} {BTN_ICON}", title: tr("Umbenennen"),
                            onclick: { let p = p.clone(); move |_| { new_name.set(p.name.clone()); renaming.set(Some(p.clone())); rename_open.set(true); } },
                            icons::Pencil { class: "size-4" }
                        }
                        button { class: "{btn(BTN_GHOST)} {BTN_ICON} text-muted-foreground hover:text-destructive", title: tr("Entfernen"),
                            onclick: { let p = p.clone(); move |_| {
                                let p = p.clone();
                                spawn(async move {
                                    if confirm(Ask::new(trf("„{0}“ entfernen?", &[&p.name]), tr("Der Drucker steht danach nicht mehr zur Auswahl.")).confirm(tr("Entfernen")).destructive()).await {
                                        match api::delete(&format!("/settings/printers/{}", p.id)).await {
                                            Ok(()) => { toast(tr("Drucker entfernt.")); version += 1; }
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
        Modal { open: rename_open, title: tr("Drucker umbenennen"),
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    let Some(p) = renaming() else { return };
                    spawn(async move {
                        match api::put::<serde_json::Value, _>(&format!("/settings/printers/{}", p.id), &json!({ "name": new_name() })).await {
                            Ok(_) => { rename_open.set(false); version += 1; }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                Field { label: tr("Name"), input { class: INPUT, required: true, maxlength: 100, value: "{new_name}", oninput: move |e| new_name.set(e.value()) } }
                div { class: "flex justify-end gap-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| rename_open.set(false), {tr("Abbrechen")} }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", {tr("Speichern")} }
                }
            }
        }
        Section { title: tr("Drucker hinzufügen"),
            div { class: "grid max-w-2xl gap-4",
                form { class: "flex gap-2",
                    onsubmit: move |e| { e.prevent_default(); add(url()); },
                    input { class: "{INPUT} font-mono", placeholder: "ipp://192.168.1.20/ipp/print", value: "{url}", oninput: move |e| url.set(e.value()) }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", disabled: adding() || url().trim().is_empty(),
                        if adding() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::Plus { class: "size-4" } }
                        {tr("Hinzufügen")}
                    }
                }
                p { class: "-mt-2 text-xs text-muted-foreground", {tr("Adresse als ipp://, ipps:// oder nur die IP. Für CUPS: ipp://server:631/printers/NAME")} }
                div { class: "flex gap-2",
                    input { class: "{INPUT} font-mono", placeholder: "192.168.0.0/24, 192.168.1.0/24", value: "{subnets}", oninput: move |e| subnets.set(e.value()) }
                    button { class: btn(BTN_OUTLINE), disabled: searching(),
                        onclick: move |_| {
                            spawn(async move {
                                searching.set(true);
                                match api::post::<Discovered, _>("/settings/printers/discover", &json!({ "subnets": subnets() })).await {
                                    Ok(d) => found.set(Some(d)),
                                    Err(e) => toast_error(e),
                                }
                                searching.set(false);
                            });
                        },
                        if searching() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::Radar { class: "size-4" } }
                        {tr("Im Netz suchen")}
                    }
                }
                if let Some(d) = found() {
                    if d.printers.is_empty() {
                        p { class: "text-sm text-muted-foreground", {trf("Kein Drucker gefunden in {0}.", &[&d.searched.join(", ")])} }
                    }
                    for p in d.printers.iter().cloned() {
                        div { class: "flex flex-wrap items-center gap-3 rounded-lg border border-dashed px-4 py-2.5",
                            div { class: "min-w-0 flex-1",
                                p { class: "font-medium", "{p.name}" }
                                p { class: "font-mono text-xs text-muted-foreground", "{p.url} · {formats_label(&p.formats)}" }
                            }
                            if rows.iter().any(|r| r.url == p.url) {
                                span { class: "text-xs text-muted-foreground", {tr("eingerichtet")} }
                            } else {
                                button { class: "{btn(BTN_OUTLINE)} {BTN_SM}", onclick: { let u = p.url.clone(); move |_| add(u.clone()) }, icons::Plus { class: "size-4" } {tr("Hinzufügen")} }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ Ordner-Eingang

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct WatchFolder {
    id: String,
    organization_id: String,
    name: String,
    path: String,
    tags: Vec<String>,
    delete_after: bool,
    enabled: bool,
    last_check_at: Option<i64>,
    last_error: String,
    imported_count: i64,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct WatchFolders {
    watch_folders: Vec<WatchFolder>,
}

#[component]
pub fn FoldersSettings(org: String) -> Element {
    let mut version = use_signal(|| 0u64);
    let list = use_resource(move || async move {
        let _ = version();
        api::get::<WatchFolders>("/settings/watch-folders").await.map(|w| w.watch_folders).unwrap_or_default()
    });
    let mut open = use_signal(|| false);
    let mut editing = use_signal(|| None::<WatchFolder>);
    let orgs = crate::state::ME.read().as_ref().map(|m| m.organizations.clone()).unwrap_or_default();
    let rows = list.read().clone().unwrap_or_default();
    rsx! {
        Section { title: tr("Ordner-Eingang"),
            description: tr("Ordner auf dem Server, deren PDFs und Bilder zur Prüfung in die Inbox kommen – z. B. der Scan-Ordner eines Multifunktionsgeräts auf dem NAS. In Docker muss der Ordner als Volume eingebunden sein. Danach wird die Datei gelöscht oder nach „nrzpaper-importiert“ verschoben."),
            actions: rsx! { button { class: btn(BTN_PRIMARY), onclick: move |_| { editing.set(None); open.set(true); }, icons::FolderPlus { class: "size-4" } {tr("Ordner hinzufügen")} } },
            if rows.is_empty() {
                p { class: "text-sm text-muted-foreground", {tr("Noch kein Ordner eingerichtet.")} }
            }
            div { class: "grid gap-3",
                for f in rows.iter().cloned() {
                    div { key: "{f.id}", class: "rounded-lg border p-4",
                        div { class: "flex flex-wrap items-center gap-3",
                            span { class: if f.enabled && f.last_error.is_empty() { "size-2 rounded-full bg-success" } else if f.enabled { "size-2 rounded-full bg-destructive" } else { "size-2 rounded-full bg-muted-foreground/40" } }
                            div { class: "min-w-0 flex-1",
                                p { class: "font-medium", "{f.name}" }
                                p { class: "truncate font-mono text-xs text-muted-foreground", "{f.path}" }
                                p { class: "text-xs text-muted-foreground",
                                    "{orgs.iter().find(|o| o.id == f.organization_id).map(|o| o.name.clone()).unwrap_or_default()}"
                                    if !f.tags.is_empty() { " · {f.tags.join(\", \")}" }
                                    " · " {trf("{0} übernommen", &[&f.imported_count])}
                                    if let Some(t) = f.last_check_at { " · " {trf("geprüft {0}", &[&util::datetime(t)])} }
                                }
                                if !f.last_error.is_empty() { p { class: "mt-1 text-xs text-destructive", "{f.last_error}" } }
                            }
                            button { class: "{btn(BTN_OUTLINE)} {BTN_SM}",
                                onclick: { let id = f.id.clone(); move |_| {
                                    let id = id.clone();
                                    spawn(async move {
                                        match api::action::<serde_json::Value>(&format!("/settings/watch-folders/{id}/scan")).await {
                                            Ok(v) => { toast(trf("{0} Datei(en) in die Inbox gelegt.", &[&v["imported"].as_i64().unwrap_or(0)])); version += 1; crate::state::refresh(); }
                                            Err(e) => toast_error(e),
                                        }
                                    });
                                } },
                                icons::RefreshCw { class: "size-4" } {tr("Jetzt einlesen")}
                            }
                            button { class: "{btn(BTN_GHOST)} {BTN_ICON}", title: tr("Bearbeiten"), onclick: { let f = f.clone(); move |_| { editing.set(Some(f.clone())); open.set(true); } }, icons::Pencil { class: "size-4" } }
                            button { class: "{btn(BTN_GHOST)} {BTN_ICON} text-muted-foreground hover:text-destructive", title: tr("Entfernen"),
                                onclick: { let f = f.clone(); move |_| {
                                    let f = f.clone();
                                    spawn(async move {
                                        if confirm(Ask::new(trf("„{0}“ entfernen?", &[&f.name]), tr("Der Ordner wird nicht mehr überwacht. Dateien darin bleiben.")).confirm(tr("Entfernen")).destructive()).await {
                                            match api::delete(&format!("/settings/watch-folders/{}", f.id)).await {
                                                Ok(()) => version += 1,
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
        FolderDialog { open, folder: editing(), org: org.clone(), on_saved: move |_| version += 1 }
    }
}

#[component]
fn FolderDialog(open: Signal<bool>, folder: Option<WatchFolder>, org: String, on_saved: EventHandler<()>) -> Element {
    let mut name = use_signal(String::new);
    let mut path = use_signal(String::new);
    let mut target = use_signal(String::new);
    let mut tags = use_signal(String::new);
    let mut delete_after = use_signal(|| false);
    let mut enabled = use_signal(|| true);
    let init = folder.clone();
    let default_org = org.clone();
    use_effect(use_reactive!(|init| {
        if open() {
            let f = init.clone().unwrap_or_default();
            name.set(f.name.clone());
            path.set(f.path.clone());
            target.set(if f.organization_id.is_empty() { default_org.clone() } else { f.organization_id.clone() });
            tags.set(f.tags.join(", "));
            delete_after.set(f.delete_after);
            enabled.set(init.is_none() || f.enabled);
        }
    }));
    let orgs = crate::state::ME.read().as_ref().map(|m| m.organizations.clone()).unwrap_or_default();
    let id = folder.as_ref().map(|f| f.id.clone());
    rsx! {
        Modal { open, title: if folder.is_some() { tr("Ordner bearbeiten") } else { tr("Ordner hinzufügen") },
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    let id = id.clone();
                    spawn(async move {
                        let body = json!({
                            "organizationId": target(), "name": name(), "path": path(),
                            "tags": tags().split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect::<Vec<_>>(),
                            "deleteAfter": delete_after(), "enabled": enabled(),
                        });
                        let res: api::ApiResult<serde_json::Value> = match &id {
                            Some(id) => api::put(&format!("/settings/watch-folders/{id}"), &body).await,
                            None => api::post("/settings/watch-folders", &body).await,
                        };
                        match res {
                            Ok(_) => { toast(tr("Ordner gespeichert.")); open.set(false); on_saved.call(()); }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                Field { label: tr("Name"), input { class: INPUT, required: true, maxlength: 100, placeholder: tr("z. B. Scans vom Drucker"), value: "{name}", oninput: move |e| name.set(e.value()) } }
                Field { label: tr("Pfad auf dem Server"), hint: tr("Absoluter Pfad, z. B. /scans – in Docker der Pfad im Container."),
                    input { class: "{INPUT} font-mono", required: true, placeholder: "/scans", value: "{path}", oninput: move |e| path.set(e.value()) }
                }
                Field { label: tr("Organisation"),
                    select { class: SELECT, onchange: move |e| target.set(e.value()),
                        for o in orgs.iter() { option { value: "{o.id}", selected: target() == o.id, "{o.name}" } }
                    }
                }
                Field { label: tr("Tags vorgeben"), hint: tr("Komma-getrennt; fehlende werden beim Freigeben angelegt"),
                    input { class: INPUT, value: "{tags}", oninput: move |e| tags.set(e.value()) }
                }
                Checkbox { checked: delete_after(), onchange: move |v| delete_after.set(v), label: tr("Datei nach dem Einlesen löschen"), hint: tr("Sonst wird sie in den Unterordner „nrzpaper-importiert“ verschoben.") }
                Toggle { checked: enabled(), onchange: move |v| enabled.set(v), label: tr("Aktiv") }
                div { class: "flex justify-end gap-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), {tr("Abbrechen")} }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", {tr("Speichern")} }
                }
            }
        }
    }
}

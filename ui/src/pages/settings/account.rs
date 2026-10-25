//! Einstellungen des eigenen Kontos: Profil & Passwort, API-Keys, Scanner, E-Mail-Eingang.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::{ApiKey, MailAccount, ScannerSettings, PERMISSIONS};
use crate::pages::scan::ScannerStatus;
use crate::state::{ME, OVERVIEW};
use crate::util;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;
use crate::i18n::{tr, trf};

#[component]
pub fn Profile() -> Element {
    let me = ME.read().clone().unwrap_or_default();
    let mut current = use_signal(String::new);
    let mut new = use_signal(String::new);
    let mut confirm_pw = use_signal(String::new);
    let mut busy = use_signal(|| false);
    rsx! {
        Section { title: tr("Profil"), description: tr("Dein Konto in dieser nrzPaper-Instanz."),
            dl { class: "grid grid-cols-[9rem_1fr] gap-y-2 text-sm",
                dt { class: "text-muted-foreground", {tr("Benutzername")} } dd { class: "font-medium", "{me.user.username}" }
                dt { class: "text-muted-foreground", {tr("Rolle")} } dd { if me.user.is_admin { {tr("Administrator")} } else { {tr("Benutzer")} } }
                dt { class: "text-muted-foreground", {tr("Organisationen")} } dd { "{me.organizations.iter().map(|o| o.name.as_str()).collect::<Vec<_>>().join(\", \")}" }
            }
        }
        Section { title: tr("Passwort ändern"), description: tr("Danach sind alle anderen Geräte abgemeldet."),
            form { class: "grid gap-4 sm:grid-cols-3 sm:items-end",
                onsubmit: move |e| {
                    e.prevent_default();
                    spawn(async move {
                        busy.set(true);
                        let res = api::put::<serde_json::Value, _>("/me/password", &json!({ "current": current(), "new": new(), "confirm": confirm_pw() })).await;
                        busy.set(false);
                        match res {
                            Ok(_) => { toast(tr("Passwort geändert. Andere Geräte sind abgemeldet.")); current.set(String::new()); new.set(String::new()); confirm_pw.set(String::new()); }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                Field { label: tr("Aktuelles Passwort"), input { class: INPUT, r#type: "password", autocomplete: "current-password", required: true, value: "{current}", oninput: move |e| current.set(e.value()) } }
                Field { label: tr("Neues Passwort"), input { class: INPUT, r#type: "password", autocomplete: "new-password", minlength: 8, required: true, value: "{new}", oninput: move |e| new.set(e.value()) } }
                Field { label: tr("Wiederholen"), input { class: INPUT, r#type: "password", autocomplete: "new-password", required: true, value: "{confirm_pw}", oninput: move |e| confirm_pw.set(e.value()) } }
                div { class: "sm:col-span-3", button { class: btn(BTN_PRIMARY), disabled: busy(), r#type: "submit", {tr("Passwort ändern")} } }
            }
        }
    }
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct Keys {
    api_keys: Vec<ApiKey>,
}

#[component]
pub fn ApiKeys() -> Element {
    let mut version = use_signal(|| 0u64);
    let keys = use_resource(move || async move {
        let _ = version();
        api::get::<Keys>("/me/api-keys").await.map(|k| k.api_keys).unwrap_or_default()
    });
    let mut open = use_signal(|| false);
    let mut shown = use_signal(|| false);
    let mut fresh = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut days = use_signal(String::new);
    let mut perms = use_signal(|| PERMISSIONS.iter().filter(|(p, _)| p.ends_with(":read")).map(|(p, _)| p.to_string()).collect::<Vec<_>>());
    rsx! {
        Section { title: tr("API-Keys"),
            description: tr("Für Skripte und andere Programme: Authorization: Bearer np_… an /api/v1. Ein Key darf nur, was angehakt ist."),
            actions: rsx! { button { class: btn(BTN_PRIMARY), onclick: move |_| { name.set(String::new()); open.set(true); }, icons::Plus { class: "size-4" } {tr("Neuer Key")} } },
            match keys.read().clone() {
                None => rsx! { Loading {} },
                Some(list) if list.is_empty() => rsx! { p { class: "text-sm text-muted-foreground", {tr("Noch keine API-Keys.")} } },
                Some(list) => rsx! {
                    div { class: "divide-y rounded-lg border",
                        for k in list {
                            div { key: "{k.id}", class: "flex flex-wrap items-center gap-3 px-4 py-3",
                                div { class: "min-w-0 flex-1",
                                    p { class: "font-medium", "{k.name} " span { class: "font-mono text-xs text-muted-foreground", {trf("{0}…", &[&(k.prefix)])} } }
                                    p { class: "mt-0.5 text-xs text-muted-foreground",
                                        {trf("angelegt {0}", &[&util::date(k.created_at)])}
                                        " · "
                                        if let Some(t) = k.last_used_at { {trf("zuletzt benutzt {0}", &[&util::datetime(t)])} } else { {tr("nie benutzt")} }
                                        if let Some(t) = k.expires_at { {trf(" · läuft ab {0}", &[&(util::date(t))])} }
                                    }
                                    div { class: "mt-1.5 flex flex-wrap gap-1",
                                        for p in k.permissions.iter() { span { class: "rounded bg-muted px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground", "{p}" } }
                                    }
                                }
                                button { class: "{btn(BTN_OUTLINE)} {BTN_SM}",
                                    onclick: { let k = k.clone(); move |_| {
                                        let k = k.clone();
                                        spawn(async move {
                                            if confirm(Ask::new({trf("API-Key „{0}“ löschen?", &[&(k.name)])}, {tr("Programme, die ihn nutzen, verlieren sofort den Zugang.")}).confirm({tr("Löschen")}).destructive()).await {
                                                match api::delete(&format!("/me/api-keys/{}", k.id)).await {
                                                    Ok(()) => { toast({tr("API-Key gelöscht.")}); version += 1; }
                                                    Err(e) => toast_error(e),
                                                }
                                            }
                                        });
                                    } },
                                    {tr("Löschen")}
                                }
                            }
                        }
                    }
                },
            }
        }
        Modal { open, title: tr("Neuer API-Key"), class: "max-w-2xl",
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    spawn(async move {
                        let d = days().parse::<u32>().ok();
                        match api::post::<serde_json::Value, _>("/me/api-keys", &json!({ "name": name(), "permissions": perms(), "expiresInDays": d })).await {
                            Ok(v) => { fresh.set(v["key"].as_str().unwrap_or_default().to_string()); open.set(false); shown.set(true); version += 1; }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                div { class: "grid gap-4 sm:grid-cols-[1fr_12rem]",
                    Field { label: tr("Name"), input { class: INPUT, required: true, maxlength: 64, placeholder: tr("z. B. Scanner im Büro"), value: "{name}", oninput: move |e| name.set(e.value()) } }
                    Field { label: tr("Gültig"),
                        select { class: SELECT, onchange: move |e| days.set(e.value()),
                            option { value: "", {tr("unbegrenzt")} } option { value: "30", {tr("30 Tage")} } option { value: "90", {tr("90 Tage")} } option { value: "365", {tr("1 Jahr")} }
                        }
                    }
                }
                div { class: "grid gap-2",
                    span { class: "text-sm font-medium", {tr("Rechte")} }
                    div { class: "grid gap-2 sm:grid-cols-2",
                        for (p , label) in PERMISSIONS.iter().copied() {
                            Checkbox { label: tr(label), hint: p.to_string(), checked: perms().iter().any(|x| x == p),
                                onchange: move |on: bool| { let mut v = perms(); v.retain(|x| x != p); if on { v.push(p.to_string()); } perms.set(v); }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), {tr("Abbrechen")} }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", disabled: name().trim().is_empty() || perms().is_empty(), {tr("Key erzeugen")} }
                }
            }
        }
        Modal { open: shown, title: tr("Dein neuer API-Key"), description: tr("Jetzt kopieren – er wird nicht wieder angezeigt."), dismissible: false,
            div { class: "flex gap-2",
                input { class: "{INPUT} font-mono", readonly: true, value: "{fresh}" }
                button { class: btn(BTN_PRIMARY), onclick: move |_| { let k = fresh(); spawn(async move { if util::copy(&k).await { toast(tr("Kopiert.")); } }); }, icons::Copy { class: "size-4" } {tr("Kopieren")} }
            }
            div { class: "mt-5 flex justify-end", button { class: btn(BTN_OUTLINE), onclick: move |_| { shown.set(false); fresh.set(String::new()); }, {tr("Fertig")} } }
        }
    }
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct ScannerEnvelope {
    scanner: ScannerSettings,
    subnets: String,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
struct Found {
    url: String,
    model: String,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
struct Discovered {
    scanners: Vec<Found>,
    subnets: Vec<String>,
}

#[component]
pub fn Scanner() -> Element {
    let mut url = use_signal(String::new);
    let mut color = use_signal(|| true);
    let mut dpi = use_signal(|| 300i64);
    let mut subnets = use_signal(String::new);
    let mut status = use_signal(ScannerStatus::default);
    let mut searching = use_signal(|| false);
    let mut found = use_signal(|| None::<Discovered>);
    use_effect(move || {
        spawn(async move {
            if let Ok(s) = api::get::<ScannerEnvelope>("/me/scanner").await {
                url.set(s.scanner.scanner_url);
                color.set(s.scanner.color);
                dpi.set(s.scanner.dpi);
                subnets.set(s.subnets);
            }
            if let Ok(s) = api::get::<ScannerStatus>("/scanner/status").await {
                status.set(s);
            }
        });
    });
    rsx! {
        Section { title: tr("Scanner"),
            description: tr("Netzwerk-Scanner mit eSCL/AirScan (z. B. Canon, HP, Epson, Brother). Gilt für dein Konto in allen Organisationen."),
            actions: rsx! {
                span { class: "inline-flex items-center gap-2 text-xs text-muted-foreground",
                    span { class: if status().ok { "size-2 rounded-full bg-success" } else { "size-2 rounded-full bg-muted-foreground/40" } }
                    if status().ok { {trf("Bereit ({0})", &[&(status().state)])} } else { "{status().state}" }
                }
            },
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    spawn(async move {
                        match api::put::<serde_json::Value, _>("/me/scanner", &json!({ "scannerUrl": url(), "color": color(), "dpi": dpi() })).await {
                            Ok(v) => {
                                url.set(v["scanner"]["scannerUrl"].as_str().unwrap_or_default().to_string());
                                toast(tr("Scanner gespeichert."));
                                if let Ok(s) = api::get::<ScannerStatus>("/scanner/status").await { status.set(s); }
                            }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                div { class: "grid gap-4 sm:grid-cols-[1fr_9rem_auto] sm:items-end",
                    Field { label: tr("Adresse"), input { class: "{INPUT} font-mono", placeholder: "http://192.168.1.50", value: "{url}", oninput: move |e| url.set(e.value()) } }
                    Field { label: tr("Auflösung"),
                        select { class: SELECT, onchange: move |e| if let Ok(v) = e.value().parse() { dpi.set(v) },
                            for d in [150i64, 300, 600] { option { value: "{d}", selected: dpi() == d, "{d} dpi" } }
                        }
                    }
                    div { class: "pb-2", Toggle { checked: color(), onchange: move |v| color.set(v), label: tr("Farbe") } }
                }
                div { class: "flex flex-wrap gap-2",
                    button { class: btn(BTN_PRIMARY), r#type: "submit", {tr("Speichern")} }
                    button { class: btn(BTN_OUTLINE), r#type: "button", disabled: searching(),
                        onclick: move |_| {
                            spawn(async move {
                                searching.set(true);
                                found.set(None);
                                match api::post::<Discovered, _>("/scanner/discover", &json!({ "subnets": subnets() })).await {
                                    Ok(d) => found.set(Some(d)),
                                    Err(e) => toast_error(e),
                                }
                                searching.set(false);
                            });
                        },
                        if searching() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::Search { class: "size-4" } }
                        {tr("Scanner im Netz suchen")}
                    }
                }
                Field { label: tr("Netze für die Suche"), hint: tr("Komma-getrennt, je /24 – z. B. 192.168.0.0/24"),
                    input { class: INPUT, value: "{subnets}", oninput: move |e| subnets.set(e.value()) }
                }
                if let Some(d) = found() {
                    if d.scanners.is_empty() {
                        Callout { text: trf("Kein Scanner gefunden in {0}.", &[&(d.subnets.join(", "))]) }
                    }
                    div { class: "grid gap-1.5",
                        for s in d.scanners {
                            button { r#type: "button", class: "flex items-center justify-between rounded-lg border px-3 py-2 text-left hover:bg-accent",
                                onclick: { let u = s.url.clone(); move |_| { url.set(u.clone()); toast(tr("Übernommen – jetzt speichern.")); } },
                                span { class: "font-medium", "{s.model}" }
                                span { class: "font-mono text-xs text-muted-foreground", "{s.url}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct MailList {
    mail_accounts: Vec<MailAccount>,
}

const PRESETS: &[(&str, &str)] = &[
    ("Gmail", "imap.gmail.com"),
    ("Outlook / Microsoft 365", "outlook.office365.com"),
    ("iCloud", "imap.mail.me.com"),
    ("GMX", "imap.gmx.net"),
    ("WEB.DE", "imap.web.de"),
    ("Bluewin", "imaps.bluewin.ch"),
    ("IONOS", "imap.ionos.de"),
];

#[component]
pub fn Mail(org: String) -> Element {
    let mut version = use_signal(|| 0u64);
    let accounts = use_resource(move || async move {
        let _ = version();
        api::get::<MailList>("/me/mail-accounts").await.map(|l| l.mail_accounts).unwrap_or_default()
    });
    let mut open = use_signal(|| false);
    let mut editing = use_signal(|| None::<MailAccount>);
    let orgs = ME.read().clone().map(|m| m.organizations).unwrap_or_default();
    rsx! {
        Section { title: tr("E-Mail-Eingang"),
            description: tr("Nur Empfang (IMAP). PDFs und Bilder aus Anhängen landen mit den vorgegebenen Tags und den Vorschlägen der KI in der Inbox. Abgeholt wird alles ab dem Einrichten; Mails werden nie gelöscht."),
            actions: rsx! { button { class: btn(BTN_PRIMARY), onclick: move |_| { editing.set(None); open.set(true); }, icons::Plus { class: "size-4" } {tr("Postfach hinzufügen")} } },
            match accounts.read().clone() {
                None => rsx! { Loading {} },
                Some(list) if list.is_empty() => rsx! { p { class: "text-sm text-muted-foreground", {tr("Noch kein Postfach eingerichtet.")} } },
                Some(list) => rsx! {
                    div { class: "grid gap-3",
                        for a in list { MailCard { key: "{a.id}", account: a.clone(), orgs: orgs.clone(), on_edit: move |a| { editing.set(Some(a)); open.set(true); }, on_change: move |_| version += 1 } }
                    }
                },
            }
        }
        MailDialog { open, account: editing(), default_org: org.clone(), on_saved: move |_| version += 1 }
    }
}

#[component]
fn MailCard(account: MailAccount, orgs: Vec<crate::models::OrgRef>, on_edit: EventHandler<MailAccount>, on_change: EventHandler<()>) -> Element {
    let mut result = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let org_name = orgs.iter().find(|o| o.id == account.organization_id).map(|o| o.name.clone()).unwrap_or_default();
    let dot = if !account.enabled { "bg-muted-foreground/40" } else if account.last_error.is_empty() { "bg-success" } else { "bg-destructive" };
    let (id_t, id_f) = (account.id.clone(), account.id.clone());
    let a_edit = account.clone();
    let a_del = account.clone();
    rsx! {
        div { class: "rounded-lg border p-4",
            div { class: "flex flex-wrap items-start gap-3",
                span { class: "mt-1.5 size-2 rounded-full {dot}" }
                div { class: "min-w-0 flex-1",
                    p { class: "font-medium", "{account.name}" }
                    p { class: "font-mono text-xs text-muted-foreground", "{account.username} @ {account.host}:{account.port} · {account.folder}" }
                    p { class: "mt-1 text-xs text-muted-foreground",
                        {trf("Ablage in {0} · alle {1} Min. · {2} Dokumente", &[&(org_name), &(account.interval_minutes), &(account.imported_count)])}
                        if let Some(t) = account.last_check_at { " · " {trf("abgerufen {0}", &[&util::datetime(t)])} }
                        if !account.enabled { " · pausiert" }
                    }
                    if !account.tags.is_empty() {
                        div { class: "mt-1.5 flex flex-wrap gap-1", for t in account.tags.iter() { span { class: "rounded-full border px-2 py-0.5 text-[11px]", "{t}" } } }
                    }
                }
                div { class: "flex gap-1",
                    button { class: "{btn(BTN_GHOST)} {BTN_ICON}", title: tr("Bearbeiten"), onclick: move |_| on_edit.call(a_edit.clone()), icons::Pencil { class: "size-4" } }
                    button { class: "{btn(BTN_GHOST)} {BTN_ICON} text-muted-foreground hover:text-destructive", title: tr("Entfernen"),
                        onclick: move |_| {
                            let a = a_del.clone();
                            spawn(async move {
                                if confirm(Ask::new(trf("Postfach „{0}“ entfernen?", &[&(a.name)]), tr("Die Mails im Postfach bleiben unberührt.")).confirm(tr("Entfernen")).destructive()).await {
                                    match api::delete(&format!("/me/mail-accounts/{}", a.id)).await {
                                        Ok(()) => { toast(tr("Postfach entfernt.")); on_change.call(()); }
                                        Err(e) => toast_error(e),
                                    }
                                }
                            });
                        },
                        icons::Trash2 { class: "size-4" }
                    }
                }
            }
            if !account.last_error.is_empty() { div { class: "mt-3", Callout { text: account.last_error.clone(), error: true } } }
            div { class: "mt-3 flex flex-wrap items-center gap-2",
                button { class: "{btn(BTN_PRIMARY)} {BTN_SM}", disabled: busy(),
                    onclick: move |_| {
                        let id = id_f.clone();
                        spawn(async move {
                            busy.set(true);
                            result.set({tr("Rufe ab …")}.into());
                            match api::action::<serde_json::Value>(&format!("/me/mail-accounts/{id}/fetch")).await {
                                Ok(v) => {
                                    let (m, d) = (v["report"]["messages"].as_i64().unwrap_or(0), v["report"]["documents"].as_i64().unwrap_or(0));
                                    result.set(if d > 0 { trf("{0} Dokument(e) aus {1} Mail(s) in die Inbox gelegt.", &[&(d), &(m)]) } else if m > 0 { trf("{0} neue Mail(s), aber keine PDFs oder Bilder.", &[&(m)]) } else { {tr("Keine neuen Mails.")}.into() });
                                    crate::state::refresh();
                                }
                                Err(e) => result.set(e.message),
                            }
                            busy.set(false);
                            on_change.call(());
                        });
                    },
                    icons::MailOpen { class: "size-4" } {tr("Jetzt abrufen")}
                }
                button { class: "{btn(BTN_OUTLINE)} {BTN_SM}", disabled: busy(),
                    onclick: move |_| {
                        let id = id_t.clone();
                        spawn(async move {
                            busy.set(true);
                            result.set({tr("Verbinde …")}.into());
                            match api::action::<serde_json::Value>(&format!("/me/mail-accounts/{id}/test")).await {
                                Ok(_) => result.set({tr("Verbindung und Anmeldung in Ordnung.")}.into()),
                                Err(e) => result.set(e.message),
                            }
                            busy.set(false);
                        });
                    },
                    {tr("Verbindung testen")}
                }
                span { class: "text-sm text-muted-foreground", "{result}" }
            }
        }
    }
}

#[component]
fn MailDialog(open: Signal<bool>, account: Option<MailAccount>, default_org: String, on_saved: EventHandler<()>) -> Element {
    let mut name = use_signal(String::new);
    let mut org = use_signal(String::new);
    let mut host = use_signal(String::new);
    let mut port = use_signal(|| 993i64);
    let mut security = use_signal(|| "tls".to_string());
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut folder = use_signal(|| "INBOX".to_string());
    let mut tags = use_signal(String::new);
    let mut interval = use_signal(|| 5i64);
    let mut enabled = use_signal(|| true);
    let mut mark_seen = use_signal(|| true);
    let init = account.clone();
    let d_org = default_org.clone();
    use_effect(use_reactive!(|init| {
        if open() {
            let a = init.clone().unwrap_or_default();
            let fresh = init.is_none();
            name.set(a.name.clone());
            org.set(if fresh { d_org.clone() } else { a.organization_id.clone() });
            host.set(a.host.clone());
            port.set(if fresh { 993 } else { a.port });
            security.set(if fresh { "tls".into() } else { a.security.clone() });
            username.set(a.username.clone());
            password.set(String::new());
            folder.set(if fresh { "INBOX".into() } else { a.folder.clone() });
            tags.set(a.tags.join(", "));
            interval.set(if fresh { 5 } else { a.interval_minutes });
            enabled.set(fresh || a.enabled);
            mark_seen.set(fresh || a.mark_seen);
        }
    }));
    let editing = account.as_ref().map(|a| a.id.clone());
    let orgs = ME.read().clone().map(|m| m.organizations).unwrap_or_default();
    let _ = OVERVIEW.read();
    let title = if account.is_some() { tr("Postfach bearbeiten") } else { tr("Postfach hinzufügen") };
    rsx! {
        Modal { open, title, class: "max-w-2xl", description: tr("Bei Gmail, iCloud und Outlook ein App-Passwort verwenden."),
            form { class: "grid gap-4",
                onsubmit: move |e| {
                    e.prevent_default();
                    let editing = editing.clone();
                    spawn(async move {
                        let body = json!({
                            "name": name(), "organizationId": org(), "host": host(), "port": port(), "security": security(),
                            "username": username(), "password": password(), "folder": folder(),
                            "tags": tags().split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect::<Vec<_>>(),
                            "intervalMinutes": interval(), "enabled": enabled(), "markSeen": mark_seen(),
                        });
                        let res: api::ApiResult<serde_json::Value> = match &editing {
                            Some(id) => api::put(&format!("/me/mail-accounts/{id}"), &body).await,
                            None => api::post("/me/mail-accounts", &body).await,
                        };
                        match res {
                            Ok(_) => { toast(tr("Postfach gespeichert.")); open.set(false); on_saved.call(()); }
                            Err(e) => toast_error(e),
                        }
                    });
                },
                div { class: "flex flex-wrap items-center gap-1.5 text-xs",
                    span { class: "text-muted-foreground", {tr("Vorlage:")} }
                    for (label , server) in PRESETS.iter().copied() {
                        button { r#type: "button", class: "rounded-full border px-2.5 py-1 hover:bg-accent",
                            onclick: move |_| { host.set(server.into()); port.set(993); security.set("tls".into()); },
                            "{label}"
                        }
                    }
                }
                div { class: "grid gap-4 sm:grid-cols-2",
                    Field { label: tr("Name"), input { class: INPUT, placeholder: tr("z. B. Privat-Mail"), value: "{name}", oninput: move |e| name.set(e.value()) } }
                    Field { label: tr("Ablage in"),
                        select { class: SELECT, onchange: move |e| org.set(e.value()),
                            for o in orgs.iter() { option { value: "{o.id}", selected: o.id == org(), "{o.name}" } }
                        }
                    }
                    Field { label: tr("IMAP-Server"), input { class: "{INPUT} font-mono", required: true, placeholder: "imap.gmail.com", value: "{host}", oninput: move |e| host.set(e.value()) } }
                    div { class: "grid grid-cols-[6rem_1fr] gap-3",
                        Field { label: tr("Port"), input { class: INPUT, r#type: "number", min: 1, max: 65535, value: "{port}", oninput: move |e| if let Ok(v) = e.value().parse() { port.set(v) } } }
                        Field { label: tr("Verschlüsselung"),
                            select { class: SELECT, onchange: move |e| security.set(e.value()),
                                for (v , l) in [("tls", "SSL/TLS".to_string()), ("starttls", "STARTTLS".to_string()), ("none", tr("keine (nur im LAN)"))] { option { value: v, selected: security() == v, "{l}" } }
                            }
                        }
                    }
                    Field { label: tr("Benutzername"), input { class: INPUT, required: true, autocomplete: "off", placeholder: "name@example.ch", value: "{username}", oninput: move |e| username.set(e.value()) } }
                    Field { label: if account.is_some() { tr("Passwort (leer = unverändert)") } else { tr("Passwort") },
                        input { class: INPUT, r#type: "password", autocomplete: "new-password", required: account.is_none(), value: "{password}", oninput: move |e| password.set(e.value()) }
                    }
                    Field { label: tr("Ordner"), input { class: "{INPUT} font-mono", value: "{folder}", oninput: move |e| folder.set(e.value()) } }
                    Field { label: tr("Tags vorgeben"), hint: tr("Komma-getrennt; fehlende werden beim Freigeben angelegt"),
                        input { class: INPUT, placeholder: tr("z. B. E-Mail, Rechnung"), value: "{tags}", oninput: move |e| tags.set(e.value()) }
                    }
                    Field { label: tr("Abruf alle … Minuten"), input { class: INPUT, r#type: "number", min: 1, max: 1440, value: "{interval}", oninput: move |e| if let Ok(v) = e.value().parse() { interval.set(v) } } }
                    div { class: "flex flex-col justify-end gap-3 pb-1",
                        Toggle { checked: enabled(), onchange: move |v| enabled.set(v), label: tr("Automatisch abrufen") }
                        Toggle { checked: mark_seen(), onchange: move |v| mark_seen.set(v), label: tr("Abgeholte Mails als gelesen markieren") }
                    }
                }
                div { class: "flex justify-end gap-2",
                    button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), {tr("Abbrechen")} }
                    button { class: btn(BTN_PRIMARY), r#type: "submit", {tr("Speichern")} }
                }
            }
        }
    }
}

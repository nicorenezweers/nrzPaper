//! Zusätze auf der Dokumentseite: Erinnerungen („Erinnere mich in 10 Tagen“) und Drucken
//! auf einem Netzwerkdrucker.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::i18n::{tr, trf};
use crate::models::*;
use crate::state;
use crate::util;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;

/// Schnellwahl in Tagen.
const QUICK: &[i64] = &[1, 3, 7, 10, 14, 30, 90];

/// Datum aus `<input type="date">` (JJJJ-MM-TT) → Unix-Sekunden, 8 Uhr Ortszeit.
fn parse_date(s: &str) -> Option<i64> {
    let mut p = s.split('-').map(|x| x.parse::<f64>().ok());
    let (y, m, d) = (p.next()??, p.next()??, p.next()??);
    let date = js_sys::Date::new_with_year_month_day_hr_min(y as u32, m as i32 - 1, d as i32, 8, 0);
    let ms = date.get_time();
    (!ms.is_nan()).then_some((ms / 1000.0) as i64)
}

#[component]
pub fn RemindersCard(org: String, doc: Document) -> Element {
    let mut version = use_signal(|| 0u64);
    let (o, i) = (org.clone(), doc.id.clone());
    let list = use_resource(use_reactive!(|o, i| async move {
        let _ = version();
        api::get::<Reminders>(&format!("/organizations/{o}/documents/{i}/reminders")).await.map(|r| r.reminders).unwrap_or_default()
    }));
    let mut note = use_signal(String::new);
    let mut custom = use_signal(String::new);
    let mut open = use_signal(|| false);
    let o2 = org.clone();
    let d2 = doc.id.clone();
    let create = use_callback(move |body: serde_json::Value| {
        let (o, d) = (o2.clone(), d2.clone());
        spawn(async move {
            let mut body = body;
            body["note"] = json!(note());
            match api::post::<serde_json::Value, _>(&format!("/organizations/{o}/documents/{d}/reminders"), &body).await {
                Ok(v) => {
                    let due = v["reminder"]["dueAt"].as_i64().unwrap_or_default();
                    toast(trf("Erinnerung am {0} gesetzt.", &[&util::date(due)]));
                    note.set(String::new());
                    custom.set(String::new());
                    open.set(false);
                    version += 1;
                    state::refresh();
                }
                Err(e) => toast_error(e),
            }
        });
    });
    let now = util::now();
    let items = list.read().clone().unwrap_or_default();
    rsx! {
        section { class: "rounded-xl border bg-card",
            div { class: "flex items-center justify-between border-b px-4 py-2.5",
                h3 { class: "flex items-center gap-2 text-sm font-semibold", icons::Bell { class: "size-4" } {tr("Erinnerungen")} }
                button { class: "{btn(BTN_GHOST)} {BTN_SM}", onclick: move |_| open.toggle(), icons::Plus { class: "size-4" } {tr("Erinnern")} }
            }
            div { class: "grid gap-2 p-4 text-sm",
                if open() {
                    div { class: "grid gap-2 rounded-lg border border-dashed p-3",
                        p { class: "text-xs text-muted-foreground", {tr("Erinnere mich in …")} }
                        div { class: "flex flex-wrap gap-1",
                            for n in QUICK.iter().copied() {
                                button { class: "{btn(BTN_OUTLINE)} h-7 px-2.5 text-xs",
                                    onclick: move |_| create.call(json!({ "days": n })),
                                    if n == 1 { {tr("1 Tag")} } else { {trf("{0} Tagen", &[&n])} }
                                }
                            }
                        }
                        div { class: "flex gap-2",
                            input { class: "{INPUT} h-8 text-xs", r#type: "date", value: "{custom}", oninput: move |e| custom.set(e.value()) }
                            button { class: "{btn(BTN_PRIMARY)} h-8 px-3 text-xs", disabled: parse_date(&custom()).is_none(),
                                onclick: move |_| if let Some(at) = parse_date(&custom()) { create.call(json!({ "dueAt": at })) },
                                {tr("Setzen")}
                            }
                        }
                        input { class: "{INPUT} h-8 text-xs", maxlength: 500, placeholder: tr("Notiz (optional), z. B. „bezahlen“"), value: "{note}", oninput: move |e| note.set(e.value()) }
                    }
                }
                if items.is_empty() && !open() {
                    p { class: "text-muted-foreground", {tr("Keine Erinnerung gesetzt.")} }
                }
                for r in items {
                    ReminderRow { key: "{r.id}", reminder: r.clone(), now, on_change: move |_| { version += 1; state::refresh(); } }
                }
            }
        }
    }
}

#[component]
fn ReminderRow(reminder: Reminder, now: i64, on_change: EventHandler<()>) -> Element {
    let r = reminder.clone();
    let done = r.done_at.is_some();
    let due = !done && r.due_at <= now;
    let (rt, rd) = (r.clone(), r.clone());
    rsx! {
        div { class: if due { "flex items-start gap-2 rounded-md border border-destructive/50 bg-destructive/10 px-2.5 py-2" } else { "flex items-start gap-2 rounded-md border px-2.5 py-2" },
            button { class: "mt-0.5", title: if done { tr("Wieder öffnen") } else { tr("Erledigt") },
                onclick: move |_| {
                    let (id, done) = (rt.id.clone(), rt.done_at.is_some());
                    spawn(async move {
                        match api::put::<serde_json::Value, _>(&format!("/reminders/{id}"), &json!({ "done": !done })).await {
                            Ok(_) => on_change.call(()),
                            Err(e) => toast_error(e),
                        }
                    });
                },
                if done { icons::CircleCheck { class: "size-4 text-success" } } else { icons::Circle { class: "size-4 text-muted-foreground" } }
            }
            div { class: "min-w-0 flex-1",
                p { class: if done { "text-muted-foreground line-through" } else { "font-medium" },
                    if due { {trf("Fällig seit {0}", &[&util::date(r.due_at)])} } else { {util::date(r.due_at)} }
                }
                if !r.note.is_empty() { p { class: "text-xs text-muted-foreground", "{r.note}" } }
            }
            button { class: "text-muted-foreground hover:text-destructive", title: tr("Löschen"),
                onclick: move |_| {
                    let id = rd.id.clone();
                    spawn(async move {
                        match api::delete(&format!("/reminders/{id}")).await {
                            Ok(()) => on_change.call(()),
                            Err(e) => toast_error(e),
                        }
                    });
                },
                icons::X { class: "size-4" }
            }
        }
    }
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Printer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub make: String,
    pub formats: Vec<String>,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
struct Printers {
    printers: Vec<Printer>,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
struct PrinterInfo {
    state: String,
    color: bool,
    duplex: bool,
    message: String,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
struct PrinterStatus {
    ok: bool,
    info: PrinterInfo,
    error: String,
}

/// Knopf „Drucken“ mit Dialog: Drucker, Kopien, beidseitig, Graustufen.
#[component]
pub fn PrintButton(org: String, doc: Document) -> Element {
    let mut open = use_signal(|| false);
    let printers = use_resource(move || async move {
        if !open() {
            return Vec::new();
        }
        api::get::<Printers>("/printers").await.map(|p| p.printers).unwrap_or_default()
    });
    let mut chosen = use_signal(|| util::storage_get("nrz-printer").unwrap_or_default());
    let mut copies = use_signal(|| 1i32);
    let mut duplex = use_signal(|| false);
    let mut gray = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let status = use_resource(move || async move {
        let id = chosen();
        if id.is_empty() || !open() {
            return None;
        }
        api::get::<PrinterStatus>(&format!("/printers/{id}/status")).await.ok()
    });
    let list = printers.read().clone().unwrap_or_default();
    // Gemerkten Drucker nur nehmen, wenn es ihn noch gibt
    if !list.is_empty() && !list.iter().any(|p| p.id == chosen()) {
        chosen.set(list[0].id.clone());
    }
    let printable = doc.kind() == "pdf" || doc.kind() == "image" || doc.mime_type.starts_with("text/");
    let (o, id, name) = (org.clone(), doc.id.clone(), doc.name.clone());
    rsx! {
        button { class: btn(BTN_OUTLINE), disabled: !printable, title: if printable { tr("Drucken") } else { tr("Diese Dateiart lässt sich nicht drucken – nur PDFs, Bilder und Text.") },
            onclick: move |_| open.set(true),
            icons::Printer { class: "size-4" } {tr("Drucken")}
        }
        Modal { open, title: tr("Drucken"), description: name.clone(),
            if printers.read().is_none() {
                Loading {}
            } else if list.is_empty() {
                Callout { text: tr("Noch kein Drucker eingerichtet – ein Administrator fügt ihn unter Einstellungen → Drucker hinzu.") }
            } else {
                form { class: "grid gap-4",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let (o, id, printer) = (o.clone(), id.clone(), chosen());
                        util::storage_set("nrz-printer", &printer);
                        spawn(async move {
                            busy.set(true);
                            let body = json!({ "printerId": printer, "copies": copies(), "duplex": duplex(), "grayscale": gray() });
                            match api::post::<serde_json::Value, _>(&format!("/organizations/{o}/documents/{id}/print"), &body).await {
                                Ok(v) => { toast(trf("An „{0}“ geschickt.", &[&v["printer"].as_str().unwrap_or("")])); open.set(false); }
                                Err(e) => toast_error(e),
                            }
                            busy.set(false);
                        });
                    },
                    Field { label: tr("Drucker"),
                        select { class: SELECT, onchange: move |e| chosen.set(e.value()),
                            for p in list.iter() { option { value: "{p.id}", selected: chosen() == p.id, "{p.name}" } }
                        }
                        match status.read().clone().flatten() {
                            None => rsx! { p { class: "mt-1 text-xs text-muted-foreground", {tr("Drucker wird geprüft …")} } },
                            Some(s) if !s.ok => rsx! { p { class: "mt-1 text-xs text-destructive", "{s.error}" } },
                            Some(s) => rsx! {
                                p { class: "mt-1 flex items-center gap-1.5 text-xs text-muted-foreground",
                                    span { class: if s.info.state == "stopped" { "size-2 rounded-full bg-destructive" } else { "size-2 rounded-full bg-success" } }
                                    {match s.info.state.as_str() { "idle" => tr("Bereit"), "processing" => tr("Druckt gerade"), "stopped" => tr("Angehalten"), _ => tr("Erreichbar") }}
                                    if !s.info.message.is_empty() { " · {s.info.message}" }
                                }
                            },
                        }
                    }
                    div { class: "grid grid-cols-[6rem_1fr] items-center gap-3",
                        Field { label: tr("Kopien"),
                            input { class: INPUT, r#type: "number", min: 1, max: 99, value: "{copies}", oninput: move |e| copies.set(e.value().parse().unwrap_or(1).clamp(1, 99)) }
                        }
                        div { class: "grid gap-2 pt-5",
                            Checkbox { checked: duplex(), onchange: move |v| duplex.set(v), label: tr("Beidseitig") }
                            Checkbox { checked: gray(), onchange: move |v| gray.set(v), label: tr("Graustufen") }
                        }
                    }
                    div { class: "flex justify-end gap-2",
                        button { class: btn(BTN_OUTLINE), r#type: "button", onclick: move |_| open.set(false), {tr("Abbrechen")} }
                        button { class: btn(BTN_PRIMARY), r#type: "submit", disabled: busy() || chosen().is_empty(),
                            if busy() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::Printer { class: "size-4" } }
                            {tr("Drucken")}
                        }
                    }
                }
            }
        }
    }
}

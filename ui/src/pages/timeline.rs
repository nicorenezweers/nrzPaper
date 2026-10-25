//! Zeitleiste rechts neben den Dokumenten: oben das heutige Datum mit fälligen und nächsten
//! Erinnerungen, darunter Tag für Tag, was eingegangen ist – und dazwischen, wie viele Tage
//! nichts kam. Filtert nach Tag, Dateityp und Text.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::i18n::{tr, trf};
use crate::models::*;
use crate::state::{self, OVERVIEW, REFRESH};
use crate::util;
use crate::Route;
use dioxus::prelude::*;
use serde_json::json;

const PAGE: u32 = 50;

/// Farben der Monate (wie Perlen an einer Kette, im Kreis).
const COLORS: &[(&str, &str)] = &[
    ("bg-sky-500", "text-sky-400"),
    ("bg-amber-500", "text-amber-400"),
    ("bg-rose-400", "text-rose-300"),
    ("bg-indigo-400", "text-indigo-300"),
    ("bg-teal-500", "text-teal-400"),
    ("bg-emerald-500", "text-emerald-400"),
];

/// Lokaler Tagesbeginn (ms) eines Zeitpunkts in Sekunden.
fn day_start(ts: i64) -> f64 {
    let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ts as f64 * 1000.0));
    d.set_hours(0);
    d.set_minutes(0);
    d.set_seconds(0);
    d.set_milliseconds(0);
    d.get_time()
}

/// Ganze Tage zwischen zwei Tagesbeginnen (Sommerzeit egal dank Rundung).
fn days_between(later: f64, earlier: f64) -> i64 {
    ((later - earlier) / 86_400_000.0).round() as i64
}

fn local(ms: f64) -> js_sys::Date {
    js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms))
}

fn day_label(ms: f64) -> String {
    let opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&opts, &"weekday".into(), &"short".into());
    let _ = js_sys::Reflect::set(&opts, &"day".into(), &"numeric".into());
    let _ = js_sys::Reflect::set(&opts, &"month".into(), &"long".into());
    local(ms).to_locale_date_string(crate::i18n::locale(), &opts).into()
}

fn type_icon(kind: &str) -> Element {
    match kind {
        "pdf" => rsx! { icons::FileText { class: "size-3.5 shrink-0 text-muted-foreground" } },
        "image" => rsx! { icons::Image { class: "size-3.5 shrink-0 text-muted-foreground" } },
        _ => rsx! { icons::File { class: "size-3.5 shrink-0 text-muted-foreground" } },
    }
}

#[component]
pub fn Timeline(org: String) -> Element {
    let mut tag = use_signal(String::new);
    let mut kind = use_signal(String::new);
    let mut text = use_signal(String::new);
    let mut docs = use_signal(Vec::<Document>::new);
    let mut total = use_signal(|| 0i64);
    let mut page = use_signal(|| 0u32);
    let mut loading = use_signal(|| false);
    let mut version = use_signal(|| 0u64);

    let query = move || {
        let mut q = Vec::new();
        let t = tag();
        if !t.is_empty() {
            q.push(if t.contains(' ') { format!("tag:\"{t}\"") } else { format!("tag:{t}") });
        }
        if !kind().is_empty() {
            q.push(format!("type:{}", kind()));
        }
        if !text().trim().is_empty() {
            q.push(text().trim().to_string());
        }
        q.join(" ")
    };

    // Neu laden bei anderem Filter, anderer Organisation oder Änderungen
    let o = org.clone();
    use_effect(use_reactive!(|o| {
        let _ = (REFRESH(), tag(), kind(), text(), version());
        let o = o.clone();
        let q = query();
        spawn(async move {
            loading.set(true);
            let url = format!("/organizations/{o}/documents?pageIndex=0&pageSize={PAGE}&searchQuery={}", api::enc(&q));
            if let Ok(l) = api::get::<DocumentList>(&url).await {
                docs.set(l.documents);
                total.set(l.documents_count);
                page.set(0);
            }
            loading.set(false);
        });
    }));
    let o2 = org.clone();
    let more = move |_| {
        let o = o2.clone();
        let q = query();
        let next = page() + 1;
        spawn(async move {
            loading.set(true);
            let url = format!("/organizations/{o}/documents?pageIndex={next}&pageSize={PAGE}&searchQuery={}", api::enc(&q));
            if let Ok(l) = api::get::<DocumentList>(&url).await {
                docs.write().extend(l.documents);
                page.set(next);
            }
            loading.set(false);
        });
    };

    let o3 = org.clone();
    let reminders = use_resource(use_reactive!(|o3| async move {
        let _ = (REFRESH(), version());
        api::get::<Reminders>(&format!("/organizations/{o3}/reminders")).await.map(|r| r.reminders).unwrap_or_default()
    }));

    let overview = OVERVIEW.read().clone().filter(|(x, _)| *x == org).map(|(_, v)| v).unwrap_or_default();
    let now = util::now();
    let today = day_start(now);

    // Nach Tag gruppieren (neueste zuerst)
    let mut list = docs();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let mut groups: Vec<(f64, Vec<Document>)> = Vec::new();
    for d in list {
        let day = day_start(d.created_at);
        match groups.last_mut() {
            Some((g, items)) if *g == day => items.push(d),
            _ => groups.push((day, vec![d])),
        }
    }
    let rems = reminders.read().clone().unwrap_or_default();
    let (due, upcoming): (Vec<Reminder>, Vec<Reminder>) = rems.into_iter().partition(|r| r.due_at <= now);
    let filtered = !tag().is_empty() || !kind().is_empty() || !text().trim().is_empty();
    let all_loaded = docs().len() as i64 >= total();

    rsx! {
        aside { class: "flex h-full min-h-0 flex-col",
            div { class: "border-b p-4",
                div { class: "flex items-center justify-between",
                    h2 { class: "flex items-center gap-2 font-semibold", icons::GitCommitVertical { class: "size-4" } {tr("Zeitleiste")} }
                    if loading() { icons::LoaderCircle { class: "size-4 animate-spin text-muted-foreground" } }
                }
                div { class: "mt-3 grid grid-cols-2 gap-2",
                    select { class: "{SELECT} h-8 text-xs", onchange: move |e| tag.set(e.value()),
                        option { value: "", selected: tag().is_empty(), {tr("Alle Tags")} }
                        for t in overview.tags.iter() { option { value: "{t.name}", selected: tag() == t.name, "{t.name}" } }
                    }
                    select { class: "{SELECT} h-8 text-xs", onchange: move |e| kind.set(e.value()),
                        option { value: "", selected: kind().is_empty(), {tr("Alle Arten")} }
                        option { value: "pdf", selected: kind() == "pdf", "PDF" }
                        option { value: "image", selected: kind() == "image", {tr("Bilder")} }
                        option { value: "text", selected: kind() == "text", {tr("Text")} }
                    }
                    input { class: "{INPUT} col-span-2 h-8 text-xs", r#type: "search", placeholder: tr("Absender, Begriff …"),
                        value: "{text}", oninput: move |e| text.set(e.value()) }
                }
            }
            div { class: "min-h-0 flex-1 overflow-y-auto overflow-x-hidden px-4 py-5",
                div { class: "relative",
                    // die Linie
                    div { class: "absolute bottom-0 left-[7px] top-2 w-0.5 rounded-full bg-border" }

                    // Heute
                    Node { color: "bg-foreground",
                        p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", {tr("Heute")} }
                        p { class: "font-semibold", "{day_label(today)}" }
                    }
                    for r in due.iter().cloned() {
                        ReminderItem { key: "{r.id}", org: org.clone(), reminder: r, due: true, on_change: move |_| { version += 1; state::refresh(); } }
                    }
                    for r in upcoming.iter().take(5).cloned() {
                        ReminderItem { key: "{r.id}", org: org.clone(), reminder: r, due: false, on_change: move |_| { version += 1; state::refresh(); } }
                    }

                    if groups.is_empty() && !loading() {
                        p { class: "ml-7 mt-4 text-sm text-muted-foreground",
                            if filtered { {tr("Nichts gefunden – Filter ändern.")} } else { {tr("Noch keine Dokumente")} }
                        }
                    }

                    {
                        let mut prev = today;
                        let mut prev_year = local(today).get_full_year();
                        groups.iter().enumerate().map(|(i, (day, items))| {
                            let gap = days_between(prev, *day) - 1;
                            let d = local(*day);
                            let year = d.get_full_year();
                            let new_year = year != prev_year;
                            prev = *day;
                            prev_year = year;
                            let (dot, txt) = COLORS[(d.get_month() as usize + year as usize * 12) % COLORS.len()];
                            let first_of_all = all_loaded && i + 1 == groups.len();
                            rsx! {
                                div { key: "{day}",
                                    if gap > 0 && !(i == 0 && gap == 0) {
                                        div { class: "relative my-1 ml-7 flex items-center gap-2 py-2 text-xs text-muted-foreground",
                                            div { class: "absolute -left-[22px] top-0 bottom-0 w-0.5 border-l-2 border-dashed border-muted-foreground/40 bg-background" }
                                            icons::MoveVertical { class: "size-3.5" }
                                            if filtered {
                                                {if gap == 1 { tr("1 Tag ohne Treffer") } else { trf("{0} Tage ohne Treffer", &[&gap]) }}
                                            } else {
                                                {if gap == 1 { tr("1 Tag ohne Dokumente") } else { trf("{0} Tage ohne Dokumente", &[&gap]) }}
                                            }
                                        }
                                    }
                                    if new_year {
                                        p { class: "my-2 ml-7 text-2xl font-bold tracking-tight {txt}", "{year}" }
                                    }
                                    Node { color: dot,
                                        p { class: "text-sm font-semibold {txt}", "{day_label(*day)}" }
                                        if first_of_all { p { class: "text-[11px] uppercase tracking-wider text-muted-foreground", {tr("Erstes Dokument")} } }
                                        div { class: "mt-1.5 grid gap-1",
                                            for doc in items.iter().cloned() {
                                                Link { key: "{doc.id}", to: Route::DocumentPage { org: org.clone(), id: doc.id.clone() },
                                                    class: "group block min-w-0 overflow-hidden rounded-md border bg-card px-2.5 py-1.5 hover:bg-accent/60",
                                                    title: "{doc.name}",
                                                    span { class: "flex min-w-0 items-center gap-1.5 text-sm",
                                                        {type_icon(doc.kind())}
                                                        span { class: "truncate group-hover:underline", "{doc.name}" }
                                                    }
                                                    if !doc.tags.is_empty() {
                                                        span { class: "mt-1 flex flex-wrap gap-1",
                                                            for t in doc.tags.iter().cloned() { TagBadge { tag: t, small: true } }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }).collect::<Vec<Element>>().into_iter()
                    }

                    if !all_loaded && !groups.is_empty() {
                        div { class: "ml-7 mt-3",
                            button { class: "{btn(BTN_OUTLINE)} {BTN_SM}", disabled: loading(), onclick: more, icons::ChevronsDown { class: "size-4" } {tr("Ältere laden")} }
                        }
                    }
                }
            }
        }
    }
}

/// Ein Punkt an der Linie mit Inhalt daneben.
#[component]
fn Node(#[props(into)] color: String, children: Element) -> Element {
    rsx! {
        div { class: "relative mb-3 min-w-0 pl-7",
            span { class: "absolute left-0 top-1 grid size-4 place-items-center rounded-full ring-4 ring-background {color}",
                span { class: "size-1.5 rounded-full bg-background" }
            }
            {children}
        }
    }
}

#[component]
fn ReminderItem(org: String, reminder: Reminder, due: bool, on_change: EventHandler<()>) -> Element {
    let r = reminder.clone();
    let (rd, rs) = (r.clone(), r.clone());
    rsx! {
        div { class: "relative mb-3 pl-7",
            span { class: if due { "absolute left-0 top-1.5 grid size-4 place-items-center rounded-full bg-destructive ring-4 ring-background" } else { "absolute left-0 top-1.5 grid size-4 place-items-center rounded-full bg-muted-foreground ring-4 ring-background" },
                icons::Bell { class: "size-2.5 text-background" }
            }
            div { class: if due { "rounded-md border border-destructive/50 bg-destructive/10 px-2.5 py-1.5" } else { "rounded-md border border-dashed px-2.5 py-1.5" },
                p { class: "text-xs text-muted-foreground",
                    if due { {tr("Erinnerung fällig")} } else { {trf("Erinnerung am {0}", &[&util::date(r.due_at)])} }
                }
                Link { class: "block truncate text-sm font-medium hover:underline", to: Route::DocumentPage { org: org.clone(), id: r.document_id.clone() }, "{r.document_name}" }
                if !r.note.is_empty() { p { class: "text-xs text-muted-foreground", "{r.note}" } }
                div { class: "mt-1 flex gap-1",
                    button { class: "{btn(BTN_GHOST)} h-7 px-2 text-xs",
                        onclick: move |_| {
                            let id = rd.id.clone();
                            spawn(async move {
                                match api::put::<serde_json::Value, _>(&format!("/reminders/{id}"), &json!({ "done": true })).await {
                                    Ok(_) => { toast(tr("Erinnerung erledigt.")); on_change.call(()); }
                                    Err(e) => toast_error(e),
                                }
                            });
                        },
                        icons::Check { class: "size-3.5" } {tr("Erledigt")}
                    }
                    if due {
                        button { class: "{btn(BTN_GHOST)} h-7 px-2 text-xs",
                            onclick: move |_| {
                                let id = rs.id.clone();
                                spawn(async move {
                                    match api::put::<serde_json::Value, _>(&format!("/reminders/{id}"), &json!({ "snoozeDays": 1 })).await {
                                        Ok(_) => { toast(tr("Auf morgen verschoben.")); on_change.call(()); }
                                        Err(e) => toast_error(e),
                                    }
                                });
                            },
                            icons::AlarmClock { class: "size-3.5" } {tr("Morgen")}
                        }
                    }
                }
            }
        }
    }
}

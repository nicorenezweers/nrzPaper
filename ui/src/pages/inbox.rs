//! Inbox: alles, was noch abgenommen werden muss – Scans, importierte PDFs und E-Mail-Anhänge.
//! Die KI schlägt Name, Tags und Metadaten vor; hier wird geprüft, geändert und freigegeben.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::{Document, InboxItem, InboxList, Tag};
use crate::pages::document::Viewer;
use crate::pages::scan::Loupe;
use crate::state;
use crate::util;
use crate::Route;
use dioxus::prelude::*;
use serde_json::json;
use std::collections::BTreeMap;

fn badge(status: &str) -> (&'static str, &'static str) {
    match status {
        "analyse" => ("KI liest …", "border-border text-muted-foreground"),
        "pruefen" => ("Prüfen", "border-primary bg-primary text-primary-foreground"),
        "import" => ("Wird abgelegt …", "border-border text-muted-foreground"),
        "fertig" => ("Abgelegt", "border-border text-muted-foreground"),
        "fehler" => ("Fehler", "border-destructive/50 text-destructive"),
        _ => ("?", ""),
    }
}

#[component]
fn StatusBadge(status: String) -> Element {
    let (label, cls) = badge(&status);
    rsx! { span { class: "shrink-0 rounded-full border px-2 py-0.5 text-[11px] font-medium {cls}", "{label}" } }
}

#[component]
pub fn Inbox(org: String, item: String) -> Element {
    let nav = use_navigator();
    let mut data = use_signal(|| None::<InboxList>);
    let mut current = use_signal(|| item.clone());
    let o = org.clone();
    let load = use_callback(move |_: ()| {
        let o = o.clone();
        spawn(async move {
            match api::get::<InboxList>(&format!("/organizations/{o}/review/items")).await {
                Ok(l) => data.set(Some(l)),
                Err(e) => toast_error(e),
            }
        });
    });
    use_effect(use_reactive!(|org| {
        let _ = org;
        load.call(());
    }));
    // Solange die KI liest oder abgelegt wird, alle 2,5 s nachsehen
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(2500).await;
            let busy = data.read().as_ref().is_some_and(|l| l.items.iter().any(InboxItem::busy));
            if busy {
                load.call(());
                state::refresh();
            }
        }
    });

    let Some(list) = data() else { return rsx! { Loading {} } };
    let open: Vec<InboxItem> = list.items.iter().filter(|i| i.status != "fertig").cloned().collect();
    let done: Vec<InboxItem> = list.items.iter().filter(|i| i.status == "fertig").cloned().collect();
    let selected = list.items.iter().find(|i| i.id == current()).cloned().or_else(|| open.first().cloned()).or_else(|| done.first().cloned());

    let row = move |i: InboxItem, active: bool| {
        rsx! {
            button { key: "{i.id}",
                class: if active { "flex w-full items-start gap-2 rounded-lg bg-accent px-2.5 py-2 text-left" } else { "flex w-full items-start gap-2 rounded-lg px-2.5 py-2 text-left hover:bg-accent/50" },
                onclick: { let id = i.id.clone(); move |_| current.set(id.clone()) },
                span { class: "mt-0.5 w-4 shrink-0 text-muted-foreground",
                    if i.busy() { icons::LoaderCircle { class: "size-4 animate-spin" } }
                    else if i.source == "mail" { icons::Mail { class: "size-4" } }
                    else if i.source == "scan" { icons::ScanLine { class: "size-4" } }
                    else { icons::FileText { class: "size-4" } }
                }
                span { class: "min-w-0 flex-1",
                    span { class: "block truncate text-sm font-medium", "{i.name}" }
                    span { class: "block truncate text-xs text-muted-foreground",
                        "{util::datetime(i.created_at)}"
                        if i.page_count > 0 { " · {i.page_count} S." }
                    }
                }
                StatusBadge { status: i.status.clone() }
            }
        }
    };

    rsx! {
        div { class: "p-4 md:p-6",
            PageHeader { title: "Inbox", description: "Scans, importierte PDFs und E-Mail-Anhänge warten hier auf die Abnahme.",
                actions: rsx! {
                    Link { class: btn(BTN_OUTLINE), to: Route::Scan { org: org.clone() }, icons::ScanLine { class: "size-4" } "Scannen" }
                }
            }
            div { class: "grid gap-5 lg:grid-cols-[17rem_minmax(0,1fr)]",
                nav { class: "space-y-1 self-start rounded-xl border bg-card p-2",
                    p { class: "px-2.5 pb-1 pt-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground", "Offen ({open.len()})" }
                    if open.is_empty() { p { class: "px-2.5 py-2 text-sm text-muted-foreground", "Nichts abzunehmen." } }
                    for i in open.iter().cloned() { {row(i.clone(), selected.as_ref().is_some_and(|s| s.id == i.id))} }
                    if !done.is_empty() {
                        p { class: "px-2.5 pb-1 pt-4 text-xs font-semibold uppercase tracking-wider text-muted-foreground", "Zuletzt abgelegt" }
                        for i in done.iter().cloned() { {row(i.clone(), selected.as_ref().is_some_and(|s| s.id == i.id))} }
                    }
                }
                if let Some(i) = selected {
                    Detail { key: "{i.id}-{i.status}", org: org.clone(), item: i, list: list.clone(),
                        on_change: move |next: Option<String>| {
                            if let Some(n) = next { current.set(n); }
                            load.call(());
                            state::refresh();
                        },
                        on_open: move |(o, d): (String, String)| { nav.push(Route::DocumentPage { org: o, id: d }); },
                    }
                } else {
                    EmptyState { title: "Alles erledigt", description: "Neue Scans, Importe und E-Mail-Anhänge landen hier.", icon: rsx! { icons::Inbox {} } }
                }
            }
        }
    }
}

#[component]
fn Detail(org: String, item: InboxItem, list: InboxList, on_change: EventHandler<Option<String>>, on_open: EventHandler<(String, String)>) -> Element {
    let base = format!("/organizations/{org}/review/{}", item.id);
    let editable = item.editable();
    let mut name = use_signal(|| item.name.clone());
    let mut target = use_signal(|| item.organization_id.clone());
    let mut tags = use_signal(|| item.tags.clone());
    let mut meta = use_signal(|| item.metadata.clone());
    let mut summary = use_signal(|| item.summary.clone());
    let mut new_tag = use_signal(String::new);
    let mut page = use_signal(|| 1i64);
    let mut busy = use_signal(|| false);

    let body = move || json!({ "name": name(), "organizationId": target(), "tags": tags(), "metadata": meta(), "summary": summary() });
    let b_save = base.clone();
    let save = use_callback(move |_: ()| {
        let b = b_save.clone();
        let payload = body();
        spawn(async move {
            if let Err(e) = api::put::<serde_json::Value, _>(&b, &payload).await {
                toast_error(e);
            }
        });
    });

    let org_tags: Vec<Tag> = if target() == org { list.tags.clone() } else { Vec::new() };
    let mut names: Vec<String> = org_tags.iter().map(|t| t.name.clone()).collect();
    for t in tags() {
        if !names.iter().any(|n| n.eq_ignore_ascii_case(&t)) {
            names.push(t);
        }
    }

    let preview = if item.status == "fertig" {
        rsx! {
            div { class: "grid min-h-[20rem] flex-1 place-items-center p-10 text-center",
                div {
                    div { class: "mx-auto grid size-12 place-items-center rounded-xl bg-muted", icons::Check { class: "size-6" } }
                    p { class: "mt-3 font-medium", "Abgelegt" }
                    if let Some(d) = item.document_id.clone() {
                        button { class: "{btn(BTN_PRIMARY)} mt-4", onclick: { let o = item.organization_id.clone(); move |_| on_open.call((o.clone(), d.clone())) }, "Dokument öffnen" }
                    }
                }
            }
        }
    } else if item.page_count > 0 {
        let src = format!("/api/v1{base}/pages/{}", page().min(item.page_count));
        rsx! {
            div { class: "relative grid flex-1 place-items-center overflow-auto bg-background p-5", Loupe { src, class: "max-h-[70vh] rounded shadow-2xl shadow-black/60" } }
            if item.page_count > 1 {
                div { class: "flex gap-3 overflow-x-auto border-t p-3",
                    for n in 1..=item.page_count {
                        button { class: if n == page() { "w-20 shrink-0 rounded-lg border-2 border-primary p-1" } else { "w-20 shrink-0 rounded-lg border-2 border-transparent p-1 hover:border-border" },
                            onclick: move |_| page.set(n),
                            img { src: "/api/v1{base}/pages/{n}", alt: "", class: "aspect-[3/4] w-full rounded object-cover" }
                            span { class: "mt-1 block text-[11px] text-muted-foreground", "Seite {n}" }
                        }
                    }
                }
            }
        }
    } else {
        let doc = Document { id: format!("inbox-{}", item.id), name: item.name.clone(), mime_type: "application/pdf".into(), ..Default::default() };
        rsx! { Viewer { url: format!("/api/v1{base}/pdf"), doc } }
    };

    let (b_ok, b_ai, b_del) = (base.clone(), base.clone(), base.clone());
    let next_open = list.items.iter().find(|i| i.status == "pruefen" && i.id != item.id).map(|i| i.id.clone());
    rsx! {
        div { class: "grid gap-5 xl:grid-cols-[minmax(0,1fr)_23rem]",
            section { class: "flex min-h-[28rem] flex-col self-start overflow-hidden rounded-xl border bg-card xl:sticky xl:top-4", {preview} }
            div { class: "space-y-4 rounded-xl border bg-card p-4 shadow-sm",
                div { class: "flex items-start justify-between gap-3",
                    h2 { class: "min-w-0 break-words font-semibold", "{item.name}" }
                    StatusBadge { status: item.status.clone() }
                }
                if !item.source_info.is_empty() {
                    p { class: "flex items-center gap-1.5 text-xs text-muted-foreground",
                        if item.source == "mail" { icons::Mail { class: "size-3.5" } }
                        "{item.source_info}"
                    }
                }
                if item.busy() {
                    p { class: "flex items-center gap-2 rounded-lg border px-3 py-2 text-sm text-muted-foreground", icons::LoaderCircle { class: "size-4 animate-spin" } "{item.step}" }
                }
                if !item.error.is_empty() { Callout { text: item.error.clone(), error: true } }
                if !item.warning.is_empty() { Callout { text: item.warning.clone() } }

                Field { label: "Name",
                    input { class: INPUT, disabled: !editable, value: "{name}", oninput: move |e| name.set(e.value()), onchange: move |_| save.call(()) }
                }
                Field { label: "Organisation",
                    select { class: SELECT, disabled: !editable,
                        onchange: move |e| { target.set(e.value()); tags.set(Vec::new()); save.call(()); },
                        for o in list.organizations.iter() { option { value: "{o.id}", selected: o.id == target(), "{o.name}" } }
                    }
                }
                div { class: "grid gap-1.5",
                    span { class: "text-sm font-medium", "Tags" }
                    div { class: "flex flex-wrap gap-1.5",
                        if names.is_empty() { span { class: "text-sm text-muted-foreground", "Keine Tags in dieser Organisation." } }
                        for n in names.iter().cloned() {
                            {
                                let known = org_tags.iter().find(|t| t.name.eq_ignore_ascii_case(&n)).cloned();
                                let on = tags().iter().any(|x| x.eq_ignore_ascii_case(&n));
                                let tag = known.clone().unwrap_or(Tag { name: n.clone(), color: "#71717a".into(), description: "neu – wird beim Freigeben angelegt".into(), ..Default::default() });
                                rsx! {
                                    TagToggle { tag, on, extra: if known.is_some() { String::new() } else { " ✱".into() },
                                        onclick: move |_| {
                                            if !editable { return; }
                                            let mut t = tags();
                                            if on { t.retain(|x| !x.eq_ignore_ascii_case(&n)); } else { t.push(n.clone()); }
                                            tags.set(t);
                                            save.call(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if editable {
                        form { class: "flex gap-2", onsubmit: move |e| {
                                e.prevent_default();
                                let n = new_tag().trim().to_string();
                                if !n.is_empty() && !tags().iter().any(|x| x.eq_ignore_ascii_case(&n)) {
                                    tags.write().push(n);
                                    save.call(());
                                }
                                new_tag.set(String::new());
                            },
                            input { class: INPUT, placeholder: "Neuer Tag (wird beim Freigeben angelegt)", value: "{new_tag}", oninput: move |e| new_tag.set(e.value()) }
                            button { class: "{btn(BTN_OUTLINE)} {BTN_ICON}", r#type: "submit", icons::Plus { class: "size-4" } }
                        }
                    }
                }
                div { class: "grid gap-1.5",
                    span { class: "text-sm font-medium", "Metadaten aus dem Dokument" }
                    div { class: "grid gap-2 sm:grid-cols-2",
                        for f in list.meta_fields.iter().cloned() {
                            label { class: "grid gap-1",
                                span { class: "text-xs text-muted-foreground", "{f.label}" }
                                input { class: "{INPUT} h-8", disabled: !editable, value: "{meta().get(&f.key).cloned().unwrap_or_default()}",
                                    oninput: { let k = f.key.clone(); move |e: FormEvent| { let mut m: BTreeMap<String, String> = meta(); m.insert(k.clone(), e.value()); meta.set(m); } },
                                    onchange: move |_| save.call(()),
                                }
                            }
                        }
                    }
                }
                Field { label: "Zusammenfassung",
                    textarea { class: TEXTAREA, rows: 3, disabled: !editable, value: "{summary}", oninput: move |e| summary.set(e.value()), onchange: move |_| save.call(()) }
                }
                if !item.model.is_empty() {
                    p { class: "flex items-center gap-1.5 text-xs text-muted-foreground", icons::Sparkles { class: "size-3.5" } "gelesen von {item.model}" }
                }
                if editable {
                    div { class: "flex flex-wrap gap-2 pt-1",
                        button { class: "{btn(BTN_PRIMARY)} flex-1", disabled: busy(),
                            onclick: move |_| {
                                let (b, payload, next) = (b_ok.clone(), body(), next_open.clone());
                                spawn(async move {
                                    busy.set(true);
                                    let res = api::post::<serde_json::Value, _>(&format!("{b}/approve"), &payload).await;
                                    busy.set(false);
                                    match res {
                                        Ok(_) => { toast(format!("„{}“ abgelegt – Texterkennung läuft.", payload["name"].as_str().unwrap_or(""))); on_change.call(next); }
                                        Err(e) => { toast_error(e); on_change.call(None); }
                                    }
                                });
                            },
                            if busy() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::Check { class: "size-4" } }
                            "Freigeben → ablegen"
                        }
                        button { class: btn(BTN_OUTLINE), title: "KI neu lesen lassen",
                            onclick: move |_| {
                                let (b, payload) = (b_ai.clone(), body());
                                spawn(async move {
                                    let _ = api::put::<serde_json::Value, _>(&b, &payload).await;
                                    match api::action::<serde_json::Value>(&format!("{b}/analyze")).await {
                                        Ok(_) => on_change.call(None),
                                        Err(e) => toast_error(e),
                                    }
                                });
                            },
                            icons::RefreshCw { class: "size-4" } "Neu analysieren"
                        }
                    }
                }
                if !item.busy() {
                    button { class: "{btn(BTN_GHOST)} {BTN_SM} text-muted-foreground hover:text-destructive",
                        onclick: move |_| {
                            let (b, done, n) = (b_del.clone(), item.status == "fertig", item.name.clone());
                            spawn(async move {
                                if !done && !confirm(Ask::new("Verwerfen?", format!("„{n}“ wird nicht abgelegt und aus der Inbox gelöscht.")).confirm("Verwerfen").destructive()).await {
                                    return;
                                }
                                match api::delete(&b).await {
                                    Ok(()) => on_change.call(Some(String::new())),
                                    Err(e) => toast_error(e),
                                }
                            });
                        },
                        icons::Trash2 { class: "size-4" }
                        if item.status == "fertig" { "Aus der Liste entfernen" } else { "Verwerfen" }
                    }
                }
            }
        }
    }
}

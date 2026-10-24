//! Dokumentliste (wie Paperless-ngx): Kacheln mit Vorschaubild oder Tabelle, Suche aus der
//! Kopfzeile, Tag-Filter, Seiten, Hochladen per Drag & Drop. Dieselbe Liste zeigt auch einen
//! Ordner (Dokumente mit einem seiner Tags).

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::{Document, DocumentList, Tag};
use crate::state::{self, OVERVIEW, REFRESH};
use crate::util;
use crate::Route;
use dioxus::prelude::*;

const PAGE_SIZE: u32 = 24;

#[component]
pub fn Documents(org: String, q: String) -> Element {
    rsx! { DocList { org, q, folder: None } }
}

#[component]
pub fn FolderView(org: String, folder: String, q: String) -> Element {
    rsx! { DocList { org, q, folder: Some(folder) } }
}

/// Grosses Vorschau-Fenster (static/preview.js) für ein Dokument.
pub fn open_preview(org: &str, doc: &Document) {
    let args = serde_json::json!({
        "url": format!("/api/v1/organizations/{org}/documents/{}/file", doc.id),
        "kind": doc.kind(),
        "name": doc.name,
        "docUrl": format!("/organizations/{org}/documents/{}", doc.id),
    });
    document::eval(&format!("import('/static/preview.js?v={}').then(m => m.openPreview({args}))", env!("CARGO_PKG_VERSION")));
}

fn file_url(org: &str, id: &str) -> String {
    format!("/api/v1/organizations/{org}/documents/{id}/file")
}

/// Suche um einen Tag-Filter ergänzen oder ihn entfernen.
fn toggle_tag(q: &str, name: &str) -> String {
    let filter = if name.contains(' ') { format!("tag:\"{name}\"") } else { format!("tag:{name}") };
    if q.contains(&filter) {
        q.replace(&filter, "").split_whitespace().collect::<Vec<_>>().join(" ")
    } else if q.trim().is_empty() {
        filter
    } else {
        format!("{} {filter}", q.trim())
    }
}

#[component]
fn DocList(org: String, q: String, folder: Option<String>) -> Element {
    let nav = use_navigator();
    let mut page = use_signal(|| 0u32);
    let mut view = use_signal(|| util::storage_get("nrz-view").unwrap_or_else(|| "grid".into()));
    let mut dragging = use_signal(|| 0i32);
    let mut uploading = use_signal(|| false);
    let mut tick = use_signal(|| 0u64);

    // Neue Suche oder anderer Ordner → erste Seite
    let key = format!("{org}|{q}|{folder:?}");
    use_effect(use_reactive!(|key| {
        let _ = key;
        page.set(0);
    }));

    let (o, qq, f) = (org.clone(), q.clone(), folder.clone());
    let list = use_resource(use_reactive!(|o, qq, f| async move {
        let _ = (REFRESH(), tick());
        let mut url = format!("/organizations/{o}/documents?pageIndex={}&pageSize={PAGE_SIZE}&searchQuery={}", page(), api::enc(&qq));
        if let Some(f) = &f {
            url.push_str(&format!("&folderId={f}"));
        }
        api::get::<DocumentList>(&url).await
    }));

    // Solange Texterkennung läuft, alle 4 s neu laden
    let busy = list.read().as_ref().and_then(|r| r.as_ref().ok()).is_some_and(|l| l.documents.iter().any(Document::busy));
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(4000).await;
            if busy {
                tick += 1;
            }
        }
    });

    let overview = OVERVIEW.read().clone().filter(|(x, _)| *x == org).map(|(_, v)| v).unwrap_or_default();
    let current_folder = folder.as_ref().and_then(|f| overview.folders.iter().find(|x| &x.id == f).cloned());
    let folder_tags: Vec<Tag> = current_folder.as_ref().map(|f| overview.tags.iter().filter(|t| f.tag_ids.contains(&t.id)).cloned().collect()).unwrap_or_default();

    let org_up = org.clone();
    let upload = use_callback(move |files: Vec<web_sys::File>| {
        if files.is_empty() {
            return;
        }
        let org = org_up.clone();
        spawn(async move {
            uploading.set(true);
            let res: api::ApiResult<serde_json::Value> = api::upload(&format!("/organizations/{org}/documents"), &files).await;
            uploading.set(false);
            match res {
                Ok(v) => {
                    let docs = if v["documents"].is_array() { v["documents"].as_array().cloned().unwrap_or_default() } else { vec![v.clone()] };
                    for d in docs {
                        let name = d["document"]["name"].as_str().unwrap_or("Dokument");
                        if d["restored"] == true {
                            toast(format!("„{name}“ aus dem Papierkorb geholt."));
                        } else {
                            toast(format!("„{name}“ hochgeladen – Texterkennung läuft."));
                        }
                    }
                    state::refresh();
                }
                Err(e) => toast_error(e),
            }
        });
    });

    let route_for = {
        let (org, folder) = (org.clone(), folder.clone());
        move |q: String| match &folder {
            Some(f) => Route::FolderView { org: org.clone(), folder: f.clone(), q },
            None => Route::Documents { org: org.clone(), q },
        }
    };
    let route_for2 = route_for.clone();

    let (title, eyebrow, description) = match &current_folder {
        Some(f) => (f.name.clone(), Some("Ordner".to_string()), None),
        None => ("Dokumente".to_string(), None, Some("PDFs und Bilder hierher ziehen – der Text wird automatisch erkannt.".to_string())),
    };

    rsx! {
        div {
            class: "relative min-h-full p-4 md:p-6",
            ondragenter: move |_| dragging += 1,
            ondragleave: move |_| dragging -= 1,
            ondragover: move |e| e.prevent_default(),
            ondrop: move |e| {
                e.prevent_default();
                dragging.set(0);
                upload.call(util::dropped_files(&e));
            },
            PageHeader {
                title,
                eyebrow,
                description,
                actions: rsx! {
                    div { class: "flex rounded-md border p-0.5",
                        button { class: if view() == "grid" { "grid size-8 place-items-center rounded-sm bg-accent" } else { "grid size-8 place-items-center rounded-sm text-muted-foreground hover:text-foreground" },
                            title: "Kacheln", onclick: move |_| { view.set("grid".into()); util::storage_set("nrz-view", "grid"); },
                            icons::LayoutGrid { class: "size-4" }
                        }
                        button { class: if view() == "list" { "grid size-8 place-items-center rounded-sm bg-accent" } else { "grid size-8 place-items-center rounded-sm text-muted-foreground hover:text-foreground" },
                            title: "Liste", onclick: move |_| { view.set("list".into()); util::storage_set("nrz-view", "list"); },
                            icons::List { class: "size-4" }
                        }
                    }
                    label { class: "{btn(BTN_PRIMARY)} cursor-pointer",
                        if uploading() { icons::LoaderCircle { class: "size-4 animate-spin" } } else { icons::Upload { class: "size-4" } }
                        "Hochladen"
                        input { r#type: "file", multiple: true, class: "hidden",
                            onchange: move |e| {
                                upload.call(util::input_files(&e));
                            }
                        }
                    }
                }
            }

            if let Some(f) = &current_folder {
                div { class: "-mt-3 mb-5 flex flex-wrap items-center gap-1.5 text-sm text-muted-foreground",
                    "Dokumente mit "
                    if folder_tags.is_empty() { span { "– noch keine Tags zugeordnet" } }
                    for t in folder_tags.iter().cloned() { TagBadge { tag: t, small: true } }
                    if f.count > 0 { span { class: "ml-1", "· {f.count} insgesamt (mit Unterordnern)" } }
                }
            }

            // Tag-Filter
            if !overview.tags.is_empty() {
                div { class: "mb-5 flex flex-wrap gap-1.5",
                    for t in overview.tags.iter().cloned() {
                        TagToggle {
                            on: q.contains(&if t.name.contains(' ') { format!("tag:\"{}\"", t.name) } else { format!("tag:{}", t.name) }),
                            extra: format!(" {}", t.documents_count),
                            tag: t.clone(),
                            onclick: {
                                let (q, name, route_for) = (q.clone(), t.name.clone(), route_for.clone());
                                move |_| { nav.push(route_for(toggle_tag(&q, &name))); }
                            }
                        }
                    }
                }
            }

            match &*list.read() {
                None => rsx! { Loading {} },
                Some(Err(e)) => rsx! { Callout { text: e.message.clone(), error: true } },
                Some(Ok(l)) if l.documents.is_empty() => rsx! {
                    if !q.is_empty() {
                        EmptyState { title: "Keine Treffer", description: format!("Für „{q}“ wurde nichts gefunden."), icon: rsx! { icons::Search {} } }
                    } else if current_folder.is_some() {
                        EmptyState { title: "Ordner ist leer", description: "Hier erscheinen Dokumente mit einem der Tags dieses Ordners.", icon: rsx! { icons::FolderOpen {} } }
                    } else {
                        EmptyState { title: "Noch keine Dokumente", description: "Dateien hierher ziehen, oben auf „Hochladen“ klicken oder scannen.", icon: rsx! { icons::FileUp {} } }
                    }
                },
                Some(Ok(l)) => {
                    let pages = ((l.documents_count as u32).div_ceil(PAGE_SIZE)).max(1);
                    rsx! {
                        p { class: "mb-3 text-sm text-muted-foreground",
                            "{l.documents_count} "
                            if l.documents_count == 1 { "Dokument" } else { "Dokumente" }
                            if !q.is_empty() { " für „{q}“" }
                            if !q.is_empty() {
                                Link { class: "ml-2 text-foreground underline-offset-4 hover:underline", to: route_for2(String::new()), "Suche löschen" }
                            }
                        }
                        if view() == "grid" {
                            div { class: "grid grid-cols-[repeat(auto-fill,minmax(12.5rem,1fr))] gap-4",
                                for d in l.documents.iter().cloned() { DocCard { key: "{d.id}", org: org.clone(), doc: d } }
                            }
                        } else {
                            DocTable { org: org.clone(), docs: l.documents.clone() }
                        }
                        if pages > 1 {
                            nav { class: "mt-6 flex items-center justify-center gap-1",
                                button { class: "{btn(BTN_GHOST)} {BTN_SM}", disabled: page() == 0, onclick: move |_| page -= 1, icons::ChevronLeft { class: "size-4" } "Zurück" }
                                span { class: "px-3 text-sm text-muted-foreground", "Seite {page() + 1} von {pages}" }
                                button { class: "{btn(BTN_GHOST)} {BTN_SM}", disabled: page() + 1 >= pages, onclick: move |_| page += 1, "Weiter" icons::ChevronRight { class: "size-4" } }
                            }
                        }
                    }
                }
            }

            if dragging() > 0 {
                div { class: "pointer-events-none fixed inset-0 z-40 grid place-items-center bg-background/85 backdrop-blur-sm",
                    div { class: "rounded-2xl border-2 border-dashed border-muted-foreground/50 px-16 py-14 text-center",
                        icons::FileUp { class: "mx-auto size-8 text-muted-foreground" }
                        p { class: "mt-3 text-lg font-semibold", "Loslassen zum Hochladen" }
                    }
                }
            }
        }
    }
}

#[component]
fn DocCard(org: String, doc: Document) -> Element {
    let to = Route::DocumentPage { org: org.clone(), id: doc.id.clone() };
    let thumb = format!("/api/v1/organizations/{org}/documents/{}/thumbnail", doc.id);
    let shown: Vec<Tag> = doc.tags.iter().take(4).cloned().collect();
    let more = doc.tags.len().saturating_sub(4);
    let d2 = doc.clone();
    let org2 = org.clone();
    rsx! {
        article { class: "group flex flex-col overflow-hidden rounded-xl border bg-card shadow-sm transition hover:border-foreground/20 hover:shadow-lg",
            Link { to: to.clone(), class: "relative block aspect-[4/5] overflow-hidden border-b bg-muted/40",
                img { src: "{thumb}", alt: "", loading: "lazy", decoding: "async", class: "h-full w-full object-cover object-top opacity-95 transition group-hover:opacity-100" }
                if !shown.is_empty() {
                    div { class: "absolute right-2 top-2 flex max-w-[85%] flex-col items-end gap-1",
                        for t in shown {
                            span { class: "inline-flex max-w-full items-center gap-1.5 truncate rounded-full border bg-background/90 px-2 py-0.5 text-[11px] font-medium shadow backdrop-blur",
                                span { class: "size-1.5 shrink-0 rounded-full", style: "background: {t.color}" }
                                "{t.name}"
                            }
                        }
                        if more > 0 { span { class: "rounded-full bg-background/90 px-2 py-0.5 text-[11px] text-muted-foreground", "+{more}" } }
                    }
                }
                if doc.busy() {
                    span { class: "absolute bottom-2 left-2 inline-flex items-center gap-1.5 rounded-full bg-background/90 px-2 py-0.5 text-[11px]",
                        icons::LoaderCircle { class: "size-3 animate-spin" } "Texterkennung …"
                    }
                }
            }
            Link { to: to.clone(), class: "flex-1 px-3 pb-2 pt-3",
                h3 { class: "line-clamp-2 break-words text-sm font-medium leading-snug", "{doc.name}" }
            }
            div { class: "space-y-1 px-3 pb-3 text-xs text-muted-foreground",
                p { class: "flex items-center gap-1.5", icons::Calendar { class: "size-3.5" } "{util::date(doc.created_at)}" }
                p { class: "flex items-center gap-1.5", icons::FileText { class: "size-3.5" }
                    "{doc.type_label()}"
                    if let Some(n) = doc.page_count { " · {n} " if n == 1 { "Seite" } else { "Seiten" } }
                    " · {util::size(doc.size)}"
                }
            }
            div { class: "grid grid-cols-3 divide-x border-t text-muted-foreground",
                Link { to, class: "grid place-items-center py-2 hover:bg-accent hover:text-foreground", title: "Öffnen und bearbeiten", icons::Pencil { class: "size-4" } }
                button { class: "grid place-items-center py-2 hover:bg-accent hover:text-foreground", title: "Vorschau", onclick: move |_| open_preview(&org2, &d2), icons::Eye { class: "size-4" } }
                a { class: "grid place-items-center py-2 hover:bg-accent hover:text-foreground", title: "Herunterladen", href: "{file_url(&org, &doc.id)}?download=true", icons::Download { class: "size-4" } }
            }
        }
    }
}

#[component]
fn DocTable(org: String, docs: Vec<Document>) -> Element {
    rsx! {
        div { class: "overflow-hidden rounded-xl border",
            table { class: "w-full text-sm",
                thead { class: "bg-muted/40 text-left text-xs uppercase tracking-wider text-muted-foreground",
                    tr {
                        th { class: "px-4 py-2.5 font-medium", "Name" }
                        th { class: "hidden px-4 py-2.5 font-medium lg:table-cell", "Tags" }
                        th { class: "hidden px-4 py-2.5 font-medium md:table-cell", "Grösse" }
                        th { class: "px-4 py-2.5 font-medium", "Hochgeladen" }
                        th { class: "w-10" }
                    }
                }
                tbody { class: "divide-y",
                    for d in docs.iter().cloned() {
                        tr { key: "{d.id}", class: "hover:bg-muted/30",
                            td { class: "px-4 py-2.5",
                                Link { to: Route::DocumentPage { org: org.clone(), id: d.id.clone() }, class: "flex items-center gap-3",
                                    span { class: "grid size-9 shrink-0 place-items-center rounded-md bg-muted text-[10px] font-semibold text-muted-foreground", "{d.type_label()}" }
                                    span { class: "min-w-0",
                                        span { class: "block truncate font-medium", "{d.name}" }
                                        span { class: "text-xs text-muted-foreground",
                                            if d.busy() { "Texterkennung läuft …" } else if let Some(n) = d.page_count { "{n} " if n == 1 { "Seite" } else { "Seiten" } }
                                        }
                                    }
                                }
                            }
                            td { class: "hidden px-4 py-2.5 lg:table-cell",
                                div { class: "flex flex-wrap gap-1", for t in d.tags.iter().cloned() { TagBadge { tag: t, small: true } } }
                            }
                            td { class: "hidden px-4 py-2.5 text-muted-foreground md:table-cell", "{util::size(d.size)}" }
                            td { class: "px-4 py-2.5 text-muted-foreground", "{util::datetime(d.created_at)}" }
                            td { class: "px-2",
                                button { class: "{btn(BTN_GHOST)} {BTN_ICON}", title: "Vorschau",
                                    onclick: { let (org, d) = (org.clone(), d.clone()); move |_| open_preview(&org, &d) },
                                    icons::Eye { class: "size-4" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

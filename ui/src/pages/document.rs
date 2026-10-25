//! Dokumentseite: grosse Vorschau (PDF.js-Viewer aus static/viewer.js) und daneben Name, Tags,
//! Eigenschaften, erkannter Text, Angaben, Freigabelinks und Verlauf. Die Angaben lassen sich
//! ausblenden – die Vorschau nutzt dann die ganze Breite.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::{Activity, Document, Member, PropertyDefinition, PropertyValue, ShareLink, Tag};
use crate::state::{self, OVERVIEW};
use crate::util;
use crate::Route;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;
use crate::i18n::{tr, trf};

#[derive(Deserialize, Default, Clone, PartialEq)]
struct DocEnvelope {
    document: Document,
}

#[component]
pub fn DocumentPage(org: String, id: String) -> Element {
    let mut version = use_signal(|| 0u64);
    let (o, i) = (org.clone(), id.clone());
    let doc = use_resource(use_reactive!(|o, i| async move {
        let _ = version();
        api::get::<DocEnvelope>(&format!("/organizations/{o}/documents/{i}")).await.map(|d| d.document)
    }));
    let mut side = use_signal(|| util::storage_get("nrz-side-hidden").as_deref() != Some("1"));

    // Solange die Texterkennung läuft, alle 4 s nachsehen
    let busy = doc.read().as_ref().and_then(|r| r.as_ref().ok()).is_some_and(Document::busy);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(4000).await;
            if busy {
                version += 1;
            }
        }
    });

    let mut reload = move || version += 1;
    let d = match &*doc.read() {
        None => return rsx! { Loading {} },
        Some(Err(e)) => return rsx! { div { class: "p-6", Callout { text: e.message.clone(), error: true } } },
        Some(Ok(d)) => d.clone(),
    };
    let in_trash = d.deleted_at.is_some();
    let file = format!("/api/v1/organizations/{org}/documents/{}/file", d.id);

    rsx! {
        div { class: "p-4 md:p-6",
            // Kopf
            div { class: "mb-4 flex flex-wrap items-start justify-between gap-4",
                div { class: "min-w-0",
                    Link { to: Route::Documents { org: org.clone(), q: String::new() }, class: "inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground",
                        icons::ArrowLeft { class: "size-4" } {tr("Dokumente")}
                    }
                    h1 { class: "mt-1 truncate text-xl font-semibold tracking-tight", "{d.name}" }
                    p { class: "mt-0.5 text-sm text-muted-foreground",
                        "{d.type_label()} · {util::size(d.size)}"
                        if let Some(n) = d.page_count { " · {n} " if n == 1 { {tr("Seite")} } else { {tr("Seiten")} } }
                        " · {util::datetime(d.created_at)}"
                    }
                }
                div { class: "flex flex-wrap gap-2",
                    button { class: "{btn(BTN_OUTLINE)} hidden xl:inline-flex", title: tr("Angaben ein- oder ausblenden"),
                        onclick: move |_| {
                            side.toggle();
                            util::storage_set("nrz-side-hidden", if side() { "0" } else { "1" });
                        },
                        icons::PanelRight { class: "size-4" }
                        if side() { {tr("Angaben ausblenden")} } else { {tr("Angaben einblenden")} }
                    }
                    a { class: btn(BTN_OUTLINE), href: "{file}?download=true", icons::Download { class: "size-4" } {tr("Herunterladen")} }
                    if !in_trash {
                        crate::pages::document_extras::PrintButton { org: org.clone(), doc: d.clone() }
                    }
                    if in_trash {
                        RestoreButtons { org: org.clone(), doc: d.clone(), on_change: move |_| reload() }
                    } else {
                        TrashButton { org: org.clone(), doc: d.clone() }
                    }
                }
            }
            if in_trash {
                div { class: "mb-4", Callout { text: tr("Dieses Dokument liegt im Papierkorb.") } }
            }

            div { class: if side() { "grid gap-5 xl:grid-cols-[minmax(0,1fr)_22rem]" } else { "grid gap-5" },
                Viewer { key: "{d.id}", url: file.clone(), doc: d.clone() }
                if side() {
                    aside { class: "space-y-4",
                        NameCard { org: org.clone(), doc: d.clone(), on_change: move |_| reload() }
                        TagsCard { org: org.clone(), doc: d.clone(), on_change: move |_| reload() }
                        if !in_trash {
                            crate::pages::document_extras::RemindersCard { org: org.clone(), doc: d.clone() }
                        }
                        PropertiesCard { org: org.clone(), doc: d.clone() }
                        ContentCard { org: org.clone(), doc: d.clone(), on_change: move |_| reload() }
                        InfoCard { doc: d.clone() }
                        ShareCard { org: org.clone(), doc: d.clone() }
                        ActivityCard { org: org.clone(), id: d.id.clone(), version: version() }
                    }
                }
            }
        }
    }
}

/// PDF-Viewer bzw. Bild/Text. Der PDF-Viewer ist das bewährte static/viewer.js (PDF.js).
#[component]
pub fn Viewer(url: String, doc: Document) -> Element {
    let id = format!("viewer-{}", doc.id);
    let kind = doc.kind();
    let mount_id = id.clone();
    use_effect(move || {
        document::eval(&format!("import('/static/viewer.js?v={}').then(m => m.mountViewer(document.getElementById('{mount_id}')))", env!("CARGO_PKG_VERSION")));
    });
    let tool = "rounded-md p-1.5 hover:bg-accent";
    match kind {
        "pdf" => rsx! {
            section { id: "{id}", "data-viewer": "", "data-kind": "pdf", "data-url": "{url}", tabindex: "-1",
                class: "flex h-[calc(100vh-10rem)] min-h-[36rem] flex-col overflow-hidden rounded-xl border bg-card outline-none",
                div { class: "flex flex-wrap items-center gap-1 border-b px-2 py-1.5 text-sm",
                    button { "data-toggle-thumbs": "", title: tr("Miniaturen"), class: tool, icons::PanelLeft { class: "size-4" } }
                    span { class: "mx-1 h-5 w-px bg-border" }
                    button { "data-prev": "", title: tr("Vorherige Seite (←)"), class: tool, icons::ChevronLeft { class: "size-4" } }
                    input { "data-page-input": "", r#type: "number", min: "1", value: "1", class: "h-7 w-12 rounded-md border bg-transparent px-1.5 text-center text-sm outline-none focus:border-ring" }
                    span { class: "text-muted-foreground", "/ " span { "data-page-count": "", {tr("–")} } }
                    button { "data-next": "", title: tr("Nächste Seite (→)"), class: tool, icons::ChevronRight { class: "size-4" } }
                    span { class: "mx-1 h-5 w-px bg-border" }
                    button { "data-zoom-out": "", title: tr("Verkleinern (−)"), class: tool, icons::ZoomOut { class: "size-4" } }
                    select { "data-zoom": "", class: "h-7 rounded-md border bg-transparent px-1.5 text-sm outline-none [&>option]:bg-popover",
                        option { value: "width", {tr("Seitenbreite")} }
                        option { value: "page", {tr("Ganze Seite")} }
                        option { value: "0.5", "50 %" }
                        option { value: "0.75", "75 %" }
                        option { value: "1", "100 %" }
                        option { value: "1.25", "125 %" }
                        option { value: "1.5", "150 %" }
                        option { value: "2", "200 %" }
                        option { value: "3", "300 %" }
                        option { value: "custom", hidden: true, "data-zoom-custom": "", {tr("–")} }
                    }
                    button { "data-zoom-in": "", title: tr("Vergrössern (+)"), class: tool, icons::ZoomIn { class: "size-4" } }
                    button { "data-rotate": "", title: tr("Drehen (R)"), class: tool, icons::RotateCw { class: "size-4" } }
                    div { class: "ml-auto flex items-center gap-1",
                        div { class: "flex items-center rounded-md border focus-within:border-ring",
                            input { "data-search": "", r#type: "search", placeholder: tr("Im Dokument suchen"), class: "h-7 w-40 bg-transparent px-2 text-sm outline-none placeholder:text-muted-foreground" }
                            span { "data-search-info": "", class: "px-1.5 text-xs text-muted-foreground" }
                            button { "data-search-prev": "", title: tr("Vorheriger Treffer"), class: "px-1.5 text-muted-foreground hover:text-foreground", "↑" }
                            button { "data-search-next": "", title: tr("Nächster Treffer (Enter)"), class: "px-1.5 text-muted-foreground hover:text-foreground", "↓" }
                        }
                        button { "data-fullscreen": "", title: tr("Vollbild"), class: tool, icons::Maximize2 { class: "size-4" } }
                    }
                }
                div { class: "flex min-h-0 flex-1",
                    aside { "data-thumbs": "", hidden: true, class: "w-40 shrink-0 space-y-1 overflow-y-auto border-r bg-card p-2" }
                    div { "data-scroller": "", tabindex: "0", class: "relative min-w-0 flex-1 overflow-auto bg-background px-6 py-5 outline-none",
                        p { "data-status": "", hidden: true, class: "mb-3 text-center text-sm text-muted-foreground" }
                        div { "data-pages": "", class: "viewer-pages" }
                    }
                }
            }
        },
        "image" => rsx! {
            section { id: "{id}", "data-viewer": "", "data-kind": "image",
                class: "flex h-[calc(100vh-10rem)] min-h-[36rem] flex-col overflow-hidden rounded-xl border bg-card",
                div { class: "flex items-center gap-1 border-b px-2 py-1.5 text-sm",
                    button { "data-zoom-out": "", class: tool, icons::ZoomOut { class: "size-4" } }
                    button { "data-zoom-reset": "", "data-zoom-custom": "", class: "min-w-16 rounded-md px-2 py-1 hover:bg-accent", "100 %" }
                    button { "data-zoom-in": "", class: tool, icons::ZoomIn { class: "size-4" } }
                    button { "data-rotate": "", title: tr("Drehen"), class: tool, icons::RotateCw { class: "size-4" } }
                    button { "data-fullscreen": "", title: tr("Vollbild"), class: "ml-auto {tool}", icons::Maximize2 { class: "size-4" } }
                }
                div { class: "image-stage grid flex-1 place-items-center overflow-auto bg-background p-6",
                    img { src: "{url}", alt: "{doc.name}", class: "max-h-full rounded shadow-2xl shadow-black/60" }
                }
            }
        },
        "text" => rsx! {
            section { class: "h-[calc(100vh-10rem)] overflow-auto rounded-xl border bg-card p-6",
                pre { class: "whitespace-pre-wrap font-mono text-sm leading-relaxed", "{doc.content}" }
            }
        },
        _ => rsx! {
            EmptyState { title: trf("Keine Vorschau für {0}", &[&(doc.mime_type)]), icon: rsx! { icons::FileText {} },
                a { class: btn(BTN_OUTLINE), href: "{url}?download=true", {tr("Herunterladen")} }
            }
        },
    }
}

#[component]
fn Card(#[props(into)] title: String, #[props(optional)] action: Option<Element>, children: Element) -> Element {
    rsx! {
        section { class: "rounded-xl border bg-card p-4 shadow-sm",
            div { class: "mb-3 flex items-center justify-between gap-2",
                h2 { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", "{title}" }
                if let Some(a) = action { {a} }
            }
            {children}
        }
    }
}

#[component]
fn TrashButton(org: String, doc: Document) -> Element {
    let nav = use_navigator();
    rsx! {
        button { class: btn(BTN_OUTLINE),
            onclick: move |_| {
                let (org, doc) = (org.clone(), doc.clone());
                spawn(async move {
                    let ask = Ask::new(tr("In den Papierkorb legen?"), trf("„{0}“ kommt in den Papierkorb und lässt sich dort wiederherstellen.", &[&(doc.name)])).confirm(tr("In den Papierkorb")).destructive();
                    if confirm(ask).await {
                        match api::delete(&format!("/organizations/{org}/documents/{}", doc.id)).await {
                            Ok(()) => {
                                toast(trf("„{0}“ liegt im Papierkorb.", &[&(doc.name)]));
                                state::refresh();
                                nav.push(Route::Documents { org, q: String::new() });
                            }
                            Err(e) => toast_error(e),
                        }
                    }
                });
            },
            icons::Trash2 { class: "size-4" }
            {tr("In den Papierkorb")}
        }
    }
}

#[component]
fn RestoreButtons(org: String, doc: Document, on_change: EventHandler<()>) -> Element {
    let nav = use_navigator();
    let (org2, doc2) = (org.clone(), doc.clone());
    rsx! {
        button { class: btn(BTN_PRIMARY),
            onclick: move |_| {
                let (org, doc) = (org.clone(), doc.clone());
                spawn(async move {
                    match api::action::<serde_json::Value>(&format!("/organizations/{org}/documents/{}/restore", doc.id)).await {
                        Ok(_) => { toast(tr("Wiederhergestellt.")); state::refresh(); on_change.call(()); }
                        Err(e) => toast_error(e),
                    }
                });
            },
            icons::Undo2 { class: "size-4" } {tr("Wiederherstellen")}
        }
        button { class: btn(BTN_DANGER),
            onclick: move |_| {
                let (org, doc) = (org2.clone(), doc2.clone());
                spawn(async move {
                    let ask = Ask::new(tr("Endgültig löschen?"), trf("„{0}“ wird unwiderruflich gelöscht – samt Text, Tags und Freigaben.", &[&(doc.name)])).confirm(tr("Endgültig löschen")).destructive();
                    if confirm(ask).await {
                        match api::delete(&format!("/organizations/{org}/documents/{}/permanent", doc.id)).await {
                            Ok(()) => { toast(tr("Endgültig gelöscht.")); nav.push(Route::Trash { org }); }
                            Err(e) => toast_error(e),
                        }
                    }
                });
            },
            icons::Trash2 { class: "size-4" } {tr("Endgültig löschen")}
        }
    }
}

#[component]
fn NameCard(org: String, doc: Document, on_change: EventHandler<()>) -> Element {
    let mut name = use_signal(|| doc.name.clone());
    let original = doc.name.clone();
    use_effect(use_reactive!(|original| name.set(original)));
    let changed = name().trim() != doc.name && !name().trim().is_empty();
    let save = move || {
        let (org, id, n) = (org.clone(), doc.id.clone(), name());
        spawn(async move {
            match api::patch::<serde_json::Value, _>(&format!("/organizations/{org}/documents/{id}"), &json!({ "name": n })).await {
                Ok(_) => { toast(tr("Umbenannt.")); state::refresh(); on_change.call(()); }
                Err(e) => toast_error(e),
            }
        });
    };
    let save2 = save.clone();
    rsx! {
        Card { title: tr("Name"),
            form { class: "flex gap-2", onsubmit: move |e| { e.prevent_default(); if changed { save2(); } },
                input { class: INPUT, value: "{name}", oninput: move |e| name.set(e.value()) }
                button { class: "{btn(BTN_OUTLINE)} {BTN_ICON}", disabled: !changed, title: tr("Speichern"), r#type: "submit", icons::Check { class: "size-4" } }
            }
        }
    }
}

#[component]
fn TagsCard(org: String, doc: Document, on_change: EventHandler<()>) -> Element {
    let mut text = use_signal(String::new);
    let all: Vec<Tag> = OVERVIEW.read().clone().filter(|(o, _)| *o == org).map(|(_, v)| v.tags).unwrap_or_default();
    let unassigned: Vec<Tag> = all.iter().filter(|t| !doc.tags.iter().any(|x| x.id == t.id)).cloned().collect();
    let list_id = format!("tags-{}", doc.id);
    let (org_add, id_add, all_add) = (org.clone(), doc.id.clone(), all.clone());
    let add = move |name: String| {
        let (org, id, all) = (org_add.clone(), id_add.clone(), all_add.clone());
        spawn(async move {
            let name = name.trim().to_string();
            if name.is_empty() {
                return;
            }
            let tag_id = match all.iter().find(|t| t.name.eq_ignore_ascii_case(&name)) {
                Some(t) => t.id.clone(),
                None => match api::post::<serde_json::Value, _>(&format!("/organizations/{org}/tags"), &json!({ "name": name })).await {
                    Ok(v) => v["tag"]["id"].as_str().unwrap_or_default().to_string(),
                    Err(e) => return toast_error(e),
                },
            };
            match api::post::<serde_json::Value, _>(&format!("/organizations/{org}/documents/{id}/tags"), &json!({ "tagId": tag_id })).await {
                Ok(_) => { text.set(String::new()); state::refresh(); on_change.call(()); }
                Err(e) => toast_error(e),
            }
        });
    };
    let add2 = add.clone();
    rsx! {
        Card { title: tr("Tags"),
            div { class: "flex flex-wrap gap-1.5",
                if doc.tags.is_empty() { span { class: "text-sm text-muted-foreground", {tr("Noch keine Tags.")} } }
                for t in doc.tags.iter().cloned() {
                    TagBadge { tag: t.clone(),
                        onremove: {
                            let (org, id) = (org.clone(), doc.id.clone());
                            move |_| {
                                let (org, id, t) = (org.clone(), id.clone(), t.clone());
                                spawn(async move {
                                    match api::delete(&format!("/organizations/{org}/documents/{id}/tags/{}", t.id)).await {
                                        Ok(()) => { state::refresh(); on_change.call(()); }
                                        Err(e) => toast_error(e),
                                    }
                                });
                            }
                        }
                    }
                }
            }
            form { class: "mt-3 flex gap-2", onsubmit: move |e| { e.prevent_default(); add2(text()); },
                input { class: INPUT, list: "{list_id}", placeholder: tr("Tag hinzufügen …"), autocomplete: "off", value: "{text}", oninput: move |e| text.set(e.value()) }
                datalist { id: "{list_id}", for t in unassigned.iter() { option { value: "{t.name}" } } }
                button { class: "{btn(BTN_OUTLINE)} {BTN_ICON}", r#type: "submit", title: tr("Hinzufügen"), icons::Plus { class: "size-4" } }
            }
            if !unassigned.is_empty() {
                div { class: "mt-2 flex flex-wrap gap-1",
                    for t in unassigned.iter().take(8).cloned() {
                        button { class: "rounded-full border border-dashed px-2 py-0.5 text-[11px] text-muted-foreground hover:border-solid hover:text-foreground",
                            onclick: { let add = add.clone(); let n = t.name.clone(); move |_| add(n.clone()) },
                            "+ {t.name}"
                        }
                    }
                }
            }
            p { class: "mt-2 text-xs text-muted-foreground", {tr("Neue Namen werden als Tag angelegt.")} }
        }
    }
}

#[derive(Deserialize, Default, Clone, PartialEq)]
struct ValuesEnvelope {
    values: Vec<PropertyValue>,
}

#[derive(Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DefsEnvelope {
    custom_properties: Vec<PropertyDefinition>,
}

#[derive(Deserialize, Default, Clone, PartialEq)]
struct MembersEnvelope {
    members: Vec<Member>,
}

#[component]
fn PropertiesCard(org: String, doc: Document) -> Element {
    let mut version = use_signal(|| 0u64);
    let (o, i) = (org.clone(), doc.id.clone());
    let data = use_resource(use_reactive!(|o, i| async move {
        let _ = version();
        let values = api::get::<ValuesEnvelope>(&format!("/organizations/{o}/documents/{i}/custom-properties")).await?;
        let defs = api::get::<DefsEnvelope>(&format!("/organizations/{o}/custom-properties")).await?;
        let members = api::get::<MembersEnvelope>(&format!("/organizations/{o}/members")).await?;
        Ok::<_, api::ApiError>((values.values, defs.custom_properties, members.members))
    }));
    let Some(Ok((values, defs, members))) = data.read().clone() else {
        return rsx! { Card { title: tr("Eigenschaften"), Loading {} } };
    };
    let nav = use_navigator();
    let org2 = org.clone();
    rsx! {
        Card { title: tr("Eigenschaften"),
            action: rsx! {
                button { class: "text-xs text-muted-foreground hover:text-foreground",
                    onclick: move |_| { nav.push(Route::Settings { org: org2.clone(), section: "properties".into() }); },
                    {tr("Verwalten")}
                }
            },
            if values.is_empty() {
                p { class: "text-sm text-muted-foreground", {tr("Noch keine Eigenschaften – unter Verwalten anlegen (z. B. Betrag, Fällig am, Status).")} }
            }
            div { class: "grid gap-3",
                for v in values.iter().cloned() {
                    PropertyEditor { key: "{v.definition_id}", org: org.clone(), doc_id: doc.id.clone(), value: v.clone(),
                        def: defs.iter().find(|d| d.id == v.definition_id).cloned().unwrap_or_default(),
                        members: members.clone(),
                        on_saved: move |_| version += 1,
                    }
                }
            }
        }
    }
}

/// Eingabe passend zum Typ der Eigenschaft; speichert beim Verlassen bzw. bei Auswahl.
#[component]
fn PropertyEditor(org: String, doc_id: String, value: PropertyValue, def: PropertyDefinition, members: Vec<Member>, on_saved: EventHandler<()>) -> Element {
    let url = format!("/organizations/{org}/documents/{doc_id}/custom-properties/{}", value.definition_id);
    let save = use_callback(move |values: Vec<String>| {
        let url = url.clone();
        spawn(async move {
            match api::put::<serde_json::Value, _>(&url, &json!({ "values": values })).await {
                Ok(_) => on_saved.call(()),
                Err(e) => toast_error(e),
            }
        });
    });
    let first = value.values.first().cloned().unwrap_or_default();
    let mut text = use_signal(|| first.clone());
    use_effect(use_reactive!(|first| text.set(first)));
    let label = rsx! { span { class: "text-xs font-medium text-muted-foreground", "{value.name}" } };
    let editor = match value.kind.as_str() {
        "boolean" => rsx! {
            Toggle { checked: first == "true", onchange: move |on: bool| save.call(vec![if on { "true".into() } else { "false".into() }]) }
        },
        "select" => rsx! {
            select { class: SELECT, onchange: move |e| save.call(vec![e.value()]),
                option { value: "", selected: first.is_empty(), "—" }
                for o in def.options.iter() { option { value: "{o.id}", selected: o.id == first, "{o.name}" } }
            }
        },
        "multi_select" => rsx! {
            div { class: "flex flex-wrap gap-1",
                for o in def.options.iter().cloned() {
                    {
                        let on = value.values.contains(&o.id);
                        let current = value.values.clone();
                        rsx! {
                            button { class: if on { "rounded-full border border-primary bg-primary px-2 py-0.5 text-xs text-primary-foreground" } else { "rounded-full border px-2 py-0.5 text-xs text-muted-foreground hover:text-foreground" },
                                onclick: move |_| {
                                    let mut next = current.clone();
                                    if on { next.retain(|x| *x != o.id); } else { next.push(o.id.clone()); }
                                    save.call(next);
                                },
                                "{o.name}"
                            }
                        }
                    }
                }
            }
        },
        "user_relation" => rsx! {
            select { class: SELECT, onchange: move |e| save.call(if e.value().is_empty() { vec![] } else { vec![e.value()] }),
                option { value: "", selected: first.is_empty(), "—" }
                for m in members.iter() { option { value: "{m.id}", selected: m.id == first, "{m.username}" } }
            }
        },
        "document_relation" => rsx! { DocRelation { org: org.clone(), self_id: doc_id.clone(), value: value.clone(), on_change: move |v| save.call(v) } },
        kind => {
            let t = match kind { "number" => "text", "date" => "date", _ => "text" };
            let mode = if kind == "number" { "decimal" } else { "text" };
            rsx! {
                input { class: INPUT, r#type: t, inputmode: mode, value: "{text}",
                    placeholder: if kind == "number" { tr("z. B. 79.90") } else { "" },
                    oninput: move |e| text.set(e.value()),
                    onchange: move |e| save.call(vec![e.value()]),
                }
            }
        }
    };
    rsx! {
        div { class: "grid gap-1.5", {label} {editor} }
    }
}

#[component]
fn DocRelation(org: String, self_id: String, value: PropertyValue, on_change: EventHandler<Vec<String>>) -> Element {
    let mut query = use_signal(String::new);
    let o = org.clone();
    let results = use_resource(move || {
        let (o, q) = (o.clone(), query());
        async move {
            if q.trim().len() < 2 {
                return Vec::new();
            }
            api::get::<crate::models::DocumentList>(&format!("/organizations/{o}/documents?pageSize=6&searchQuery={}", api::enc(&q))).await.map(|l| l.documents).unwrap_or_default()
        }
    });
    rsx! {
        div { class: "grid gap-1.5",
            div { class: "flex flex-wrap gap-1",
                for (id , name) in value.values.iter().cloned().zip(value.display.iter().cloned()) {
                    span { class: "inline-flex items-center gap-1 rounded-md border bg-secondary/60 px-2 py-0.5 text-xs",
                        Link { to: Route::DocumentPage { org: org.clone(), id: id.clone() }, class: "hover:underline", "{name}" }
                        button { class: "text-muted-foreground hover:text-foreground",
                            onclick: { let current = value.values.clone(); move |_| { let mut n = current.clone(); n.retain(|x| *x != id); on_change.call(n); } },
                            icons::X { class: "size-3" }
                        }
                    }
                }
            }
            input { class: INPUT, placeholder: tr("Dokument suchen …"), value: "{query}", oninput: move |e| query.set(e.value()) }
            for d in results.read().clone().unwrap_or_default().into_iter().filter(|d| d.id != self_id && !value.values.contains(&d.id)) {
                button { class: "truncate rounded-md px-2 py-1 text-left text-sm hover:bg-accent",
                    onclick: { let current = value.values.clone(); move |_| { let mut n = current.clone(); n.push(d.id.clone()); query.set(String::new()); on_change.call(n); } },
                    "+ {d.name}"
                }
            }
        }
    }
}

#[component]
fn ContentCard(org: String, doc: Document, on_change: EventHandler<()>) -> Element {
    let mut content = use_signal(|| doc.content.clone());
    let original = doc.content.clone();
    use_effect(use_reactive!(|original| content.set(original)));
    let (org2, id2) = (org.clone(), doc.id.clone());
    let status_line = match &doc.ocr_error {
        Some(e) => {trf("{0} – {1}", &[&(crate::models::ocr_label(&doc.ocr_status)), &(e)])},
        None => crate::models::ocr_label(&doc.ocr_status).to_string(),
    };
    rsx! {
        Card { title: tr("Erkannter Text"),
            action: rsx! {
                button { class: "inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground", title: tr("Texterkennung neu starten"),
                    onclick: move |_| {
                        let (org, id) = (org2.clone(), id2.clone());
                        spawn(async move {
                            match api::action::<serde_json::Value>(&format!("/organizations/{org}/documents/{id}/reprocess")).await {
                                Ok(_) => { toast(tr("Texterkennung neu gestartet.")); on_change.call(()); }
                                Err(e) => toast_error(e),
                            }
                        });
                    },
                    icons::RefreshCw { class: "size-3.5" } {tr("Neu erkennen")}
                }
            },
            p { class: "mb-2 flex items-center gap-1.5 text-xs text-muted-foreground",
                if doc.busy() { icons::LoaderCircle { class: "size-3.5 animate-spin" } }
                "{status_line}"
            }
            textarea { class: "{TEXTAREA} min-h-40 font-mono text-xs leading-relaxed", placeholder: tr("Kein Text erkannt."), value: "{content}", oninput: move |e| content.set(e.value()) }
            if content() != doc.content {
                button { class: "{btn(BTN_OUTLINE)} {BTN_SM} mt-2",
                    onclick: move |_| {
                        let (org, id, c) = (org.clone(), doc.id.clone(), content());
                        spawn(async move {
                            match api::patch::<serde_json::Value, _>(&format!("/organizations/{org}/documents/{id}"), &json!({ "content": c })).await {
                                Ok(_) => { toast(tr("Text gespeichert.")); on_change.call(()); }
                                Err(e) => toast_error(e),
                            }
                        });
                    },
                    {tr("Text speichern")}
                }
            }
        }
    }
}

#[component]
fn InfoCard(doc: Document) -> Element {
    let source = match doc.source.as_str() {
        "api" => {tr("API")},
        "ingestion" => {tr("Eingangsordner")},
        _ => {tr("Oberfläche")},
    };
    rsx! {
        Card { title: tr("Angaben"),
            dl { class: "grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-sm",
                dt { class: "text-muted-foreground", {tr("Hochgeladen")} } dd { "{util::datetime(doc.created_at)}" }
                dt { class: "text-muted-foreground", {tr("Quelle")} } dd { "{source}" }
                dt { class: "text-muted-foreground", {tr("Typ")} } dd { class: "truncate", "{doc.mime_type}" }
                dt { class: "text-muted-foreground", {tr("Grösse")} } dd { "{util::size(doc.size)}" }
                dt { class: "text-muted-foreground", {tr("Original")} } dd { class: "truncate", title: "{doc.original_name}", "{doc.original_name}" }
                dt { class: "text-muted-foreground", {tr("SHA-256")} } dd { class: "truncate font-mono text-xs text-muted-foreground", title: "{doc.sha256}", "{doc.sha256}" }
            }
        }
    }
}

#[derive(Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SharesEnvelope {
    share_links: Vec<ShareLink>,
}

#[component]
fn ShareCard(org: String, doc: Document) -> Element {
    let mut version = use_signal(|| 0u64);
    let mut days = use_signal(|| "7".to_string());
    let mut fresh = use_signal(|| None::<String>);
    let (o, i) = (org.clone(), doc.id.clone());
    let links = use_resource(use_reactive!(|o, i| async move {
        let _ = version();
        api::get::<SharesEnvelope>(&format!("/organizations/{o}/documents/{i}/share-links")).await.map(|s| s.share_links).unwrap_or_default()
    }));
    let now = util::now();
    let (org_c, id_c) = (org.clone(), doc.id.clone());
    rsx! {
        Card { title: tr("Freigabe"),
            if let Some(link) = fresh() {
                div { class: "mb-3 rounded-lg border bg-muted/40 p-3",
                    p { class: "text-xs", {tr("Neuer Link – jetzt kopieren, er wird nicht wieder angezeigt:")} }
                    div { class: "mt-2 flex gap-2",
                        input { class: "{INPUT} font-mono text-xs", readonly: true, value: "{link}" }
                        button { class: "{btn(BTN_PRIMARY)} {BTN_SM}",
                            onclick: move |_| { let l = link.clone(); spawn(async move { if util::copy(&l).await { toast(tr("Link kopiert.")); } }); },
                            icons::Copy { class: "size-4" }
                        }
                    }
                }
            }
            ul { class: "space-y-1.5 text-sm",
                for l in links.read().clone().unwrap_or_default() {
                    {
                        let active = l.revoked_at.is_none() && l.expires_at.is_none_or(|e| e > now);
                        let (org, id, lid) = (org.clone(), doc.id.clone(), l.id.clone());
                        rsx! {
                            li { class: "flex items-center gap-2",
                                span { class: if active { "min-w-0 flex-1 truncate" } else { "min-w-0 flex-1 truncate text-muted-foreground line-through" },
                                    {trf("Link vom {0}", &[&(util::date(l.created_at))])}
                                    span { class: "text-xs text-muted-foreground",
                                        {trf(" · {0}× geöffnet", &[&(l.view_count)])}
                                        if let Some(e) = l.expires_at { " · " {trf("bis {0}", &[&util::date(e)])} }
                                    }
                                }
                                if l.revoked_at.is_none() {
                                    button { class: "text-xs text-muted-foreground hover:text-foreground",
                                        onclick: move |_| {
                                            let (org, id, lid) = (org.clone(), id.clone(), lid.clone());
                                            spawn(async move {
                                                if confirm(Ask::new({tr("Freigabelink widerrufen?")}, {tr("Wer den Link hat, kann das Dokument danach nicht mehr öffnen.")}).confirm({tr("Widerrufen")}).destructive()).await {
                                                    match api::delete(&format!("/organizations/{org}/documents/{id}/share-links/{lid}")).await {
                                                        Ok(()) => { toast({tr("Link widerrufen.")}); version += 1; }
                                                        Err(e) => toast_error(e),
                                                    }
                                                }
                                            });
                                        },
                                        {tr("Widerrufen")}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if doc.deleted_at.is_none() {
                div { class: "mt-3 flex gap-2",
                    select { class: SELECT, value: "{days}", onchange: move |e| days.set(e.value()),
                        option { value: "7", {tr("7 Tage gültig")} }
                        option { value: "30", {tr("30 Tage gültig")} }
                        option { value: "365", {tr("1 Jahr gültig")} }
                        option { value: "", {tr("unbefristet")} }
                    }
                    button { class: "{btn(BTN_OUTLINE)} shrink-0",
                        onclick: move |_| {
                            let (org, id) = (org_c.clone(), id_c.clone());
                            let d = days().parse::<u32>().ok();
                            spawn(async move {
                                match api::post::<serde_json::Value, _>(&format!("/organizations/{org}/documents/{id}/share-links"), &json!({ "expiresInDays": d })).await {
                                    Ok(v) => {
                                        fresh.set(Some(util::absolute(v["path"].as_str().unwrap_or_default())));
                                        version += 1;
                                    }
                                    Err(e) => toast_error(e),
                                }
                            });
                        },
                        icons::Share2 { class: "size-4" } {tr("Link erstellen")}
                    }
                }
                p { class: "mt-2 text-xs text-muted-foreground", {tr("Wer den Link hat, sieht das Dokument ohne Anmeldung.")} }
            }
        }
    }
}

#[derive(Deserialize, Default, Clone, PartialEq)]
struct ActivityEnvelope {
    activities: Vec<Activity>,
}

fn activity_label(a: &Activity) -> String {
    let tag = a.data["tag"].as_str().unwrap_or("");
    match a.event.as_str() {
        "created" => {tr("Hochgeladen")}.into(),
        "renamed" => {trf("Umbenannt in „{0}“", &[&(a.data["to"].as_str().unwrap_or(""))])},
        "content_updated" => {tr("Text bearbeitet")}.into(),
        "tag_added" => {trf("Tag „{0}“ hinzugefügt", &[&(tag)])},
        "tag_removed" => {trf("Tag „{0}“ entfernt", &[&(tag)])},
        "deleted" => {tr("In den Papierkorb gelegt")}.into(),
        "restored" => {tr("Wiederhergestellt")}.into(),
        "shared" => {tr("Freigabelink erstellt")}.into(),
        "share_revoked" => {tr("Freigabelink widerrufen")}.into(),
        "reviewed" => match a.data["model"].as_str().filter(|m| !m.is_empty()) {
            Some(m) => {trf("Geprüft und abgelegt (KI: {0})", &[&(m)])},
            None => {tr("Geprüft und abgelegt")}.into(),
        },
        "printed" => trf("Gedruckt auf „{0}“", &[&a.data["printer"].as_str().unwrap_or("")]),
        other => other.to_string(),
    }
}

#[component]
fn ActivityCard(org: String, id: String, version: u64) -> Element {
    let (o, i) = (org.clone(), id.clone());
    let list = use_resource(use_reactive!(|o, i, version| async move {
        let _ = version;
        api::get::<ActivityEnvelope>(&format!("/organizations/{o}/documents/{i}/activity?pageSize=30")).await.map(|a| a.activities).unwrap_or_default()
    }));
    rsx! {
        Card { title: tr("Verlauf"),
            ol { class: "space-y-2 text-sm",
                for a in list.read().clone().unwrap_or_default() {
                    li { class: "flex justify-between gap-3",
                        span {
                            "{activity_label(&a)}"
                            if let Some(u) = &a.username { span { class: "text-muted-foreground", " · {u}" } }
                        }
                        span { class: "shrink-0 text-xs text-muted-foreground", "{util::datetime(a.created_at)}" }
                    }
                }
            }
        }
    }
}

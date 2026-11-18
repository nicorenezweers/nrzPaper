//! Scannen (aus nrzDocsScanner): Seiten vom Netzwerk-Scanner oder als Fotos, Lupe, Reihenfolge
//! per Drag & Drop, Tags vorgeben – dann als PDF in die Inbox, wo die KI Name, Tags und
//! Metadaten vorschlägt.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::{ScanState, ScannerSettings};
use crate::state;
use crate::util;
use crate::Route;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;
use crate::i18n::{tr, trf};

#[derive(Deserialize, Clone, PartialEq, Default)]
pub struct ScannerStatus {
    pub ok: bool,
    pub state: String,
    pub url: String,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
struct ScannerEnvelope {
    scanner: ScannerSettings,
}

/// Bild mit Lupe beim Überfahren (rust-ui-Stil, in Rust statt JavaScript).
#[component]
pub fn Loupe(#[props(into)] src: String, #[props(into, optional)] class: Option<String>) -> Element {
    let mut size = use_signal(|| (0.0f64, 0.0f64));
    let mut pos = use_signal(|| None::<(f64, f64)>);
    let mut el = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let zoom = 2.6;
    let half = 120.0;
    let lens = pos().map(|(x, y)| {
        let (w, h) = size();
        format!(
            "display:block;left:{}px;top:{}px;background-image:url('{src}');background-size:{}px {}px;background-position:{}px {}px",
            x - half,
            y - half,
            w * zoom,
            h * zoom,
            half - x * zoom,
            half - y * zoom
        )
    });
    rsx! {
        div { class: "relative inline-block",
            img {
                src: "{src}",
                alt: "",
                class: class.unwrap_or_else(|| "max-h-[62vh] rounded shadow-2xl shadow-black/60".into()),
                onmounted: move |e| el.set(Some(e.data())),
                onmouseenter: move |_| async move {
                    if let Some(m) = el() {
                        if let Ok(r) = m.get_client_rect().await {
                            size.set((r.width(), r.height()));
                        }
                    }
                },
                onmousemove: move |e| {
                    let p = e.element_coordinates();
                    pos.set(Some((p.x, p.y)));
                },
                onmouseleave: move |_| pos.set(None),
            }
            if let Some(style) = lens {
                div { class: "loupe", style: "{style}" }
            }
        }
    }
}

#[component]
pub fn Scan(org: String) -> Element {
    let nav = use_navigator();
    let base = format!("/organizations/{org}/scan");
    let mut data = use_signal(ScanState::default);
    let mut selected = use_signal(|| None::<String>);
    let mut scanning = use_signal(|| false);
    let mut status = use_signal(ScannerStatus::default);
    let mut color = use_signal(|| true);
    let mut dpi = use_signal(|| 300i64);
    let mut name = use_signal(String::new);
    let mut drag_id = use_signal(|| None::<String>);
    let mut drop_before = use_signal(|| None::<String>);
    let mut uploading = use_signal(|| false);
    let mut viewing = use_signal(|| None::<String>);

    // Laden: Sitzung, Scanner-Einstellungen, Status
    let b0 = base.clone();
    use_effect(move || {
        let _ = state::SCAN_TICK();
        let b = b0.clone();
        spawn(async move {
            if let Ok(s) = api::get::<ScanState>(&format!("{b}/session")).await {
                name.set(s.session.name.clone());
                data.set(s);
            }
            if let Ok(s) = api::get::<ScannerEnvelope>("/me/scanner").await {
                color.set(util::storage_get("nrz-scan-color").map(|v| v == "true").unwrap_or(s.scanner.color));
                dpi.set(util::storage_get("nrz-scan-dpi").and_then(|v| v.parse().ok()).unwrap_or(s.scanner.dpi));
            }
        });
    });
    use_future(move || async move {
        loop {
            if let Ok(s) = api::get::<ScannerStatus>("/scanner/status").await {
                status.set(s);
            }
            gloo_timers::future::TimeoutFuture::new(30_000).await;
        }
    });

    let pages = data().session.pages.clone();
    let n = pages.len();
    let sel = selected().filter(|s| pages.contains(s)).or_else(|| pages.last().cloned());
    let idx = sel.as_ref().and_then(|s| pages.iter().position(|p| p == s)).unwrap_or(0);

    let b_upd = base.clone();
    let update = use_callback(move |patch: serde_json::Value| {
        let b = b_upd.clone();
        spawn(async move {
            match api::put::<ScanState, _>(&format!("{b}/session"), &patch).await {
                Ok(s) => data.set(s),
                Err(e) => toast_error(e),
            }
        });
    });
    let move_page = use_callback(move |(id, to): (String, usize)| {
        let mut order: Vec<String> = data().session.pages.into_iter().filter(|p| *p != id).collect();
        let to = to.min(order.len());
        order.insert(to, id.clone());
        selected.set(Some(id));
        update.call(json!({ "pages": order }));
    });
    let b_del = base.clone();
    let remove_page = use_callback(move |id: String| {
        let b = b_del.clone();
        spawn(async move {
            match api::send::<ScanState, _>("DELETE", &format!("{b}/pages/{id}"), &json!({})).await {
                Ok(s) => data.set(s),
                Err(e) => toast_error(e),
            }
        });
    });

    let b_scan = base.clone();
    let scan_now = use_callback(move |_: ()| {
        if scanning() {
            return;
        }
        let b = b_scan.clone();
        spawn(async move {
            scanning.set(true);
            match api::post::<ScanState, _>(&format!("{b}/page"), &json!({ "color": color(), "dpi": dpi() })).await {
                Ok(s) => {
                    selected.set(s.session.pages.last().cloned());
                    toast(trf("Seite {0} gescannt.", &[&(s.session.pages.len())]));
                    data.set(s);
                }
                Err(e) => toast_error(e),
            }
            scanning.set(false);
            if let Ok(s) = api::get::<ScannerStatus>("/scanner/status").await {
                status.set(s);
            }
        });
    });

    let (b_fin, org_fin) = (base.clone(), org.clone());
    let finish = use_callback(move |_: ()| {
        let (b, org) = (b_fin.clone(), org_fin.clone());
        spawn(async move {
            let _ = api::put::<ScanState, _>(&format!("{b}/session"), &json!({ "name": name() })).await;
            match api::action::<serde_json::Value>(&format!("{b}/finish")).await {
                Ok(v) => {
                    toast(tr("Gespeichert – die KI liest das Dokument …"));
                    state::refresh();
                    nav.push(Route::Inbox { org, item: v["item"]["id"].as_str().unwrap_or_default().to_string() });
                }
                Err(e) => toast_error(e),
            }
        });
    });
    let b_clear = base.clone();
    let discard = use_callback(move |_: ()| {
        let b = b_clear.clone();
        spawn(async move {
            if confirm(Ask::new(trf("Alle {0} Seiten verwerfen?", &[&(n)]), tr("Die gescannten Seiten werden gelöscht.")).confirm(tr("Verwerfen")).destructive()).await {
                match api::send::<ScanState, _>("DELETE", &format!("{b}/session"), &json!({})).await {
                    Ok(s) => {
                        name.set(String::new());
                        data.set(s);
                    }
                    Err(e) => toast_error(e),
                }
            }
        });
    });
    // Handy: Fotos gedreht und verkleinert hochladen (static/mobile.js)
    let b_mob = base.clone();
    let mobile_photos = use_callback(move |input_id: &'static str| {
        let b = b_mob.clone();
        spawn(async move {
            uploading.set(true);
            match crate::mobile::upload_photos::<ScanState>(input_id, &format!("{b}/photos")).await {
                Ok(Some(s)) => {
                    selected.set(s.session.pages.last().cloned());
                    data.set(s);
                }
                Ok(None) => {}
                Err(e) => toast_error(e),
            }
            uploading.set(false);
        });
    });
    let b_photo = base.clone();
    let add_photos = move |e: FormEvent| {
        let files = util::input_files(&e);
        let b = b_photo.clone();
        spawn(async move {
            match api::upload::<ScanState>(&format!("{b}/photos"), &files).await {
                Ok(s) => data.set(s),
                Err(e) => toast_error(e),
            }
        });
    };
    let org_pdf = org.clone();
    let import_pdfs = use_callback(move |e: FormEvent| {
        let files = util::input_files(&e);
        let org = org_pdf.clone();
        spawn(async move {
            uploading.set(true);
            match api::upload::<serde_json::Value>(&format!("/organizations/{org}/review/import"), &files).await {
                Ok(v) => {
                    let first = v["items"][0]["id"].as_str().unwrap_or_default().to_string();
                    toast(tr("PDF zur Prüfung – die KI liest …"));
                    state::refresh();
                    nav.push(Route::Inbox { org, item: first });
                }
                Err(e) => toast_error(e),
            }
            uploading.set(false);
        });
    });

    let tags = data().tags.clone();
    let hints = data().session.hint_tags.clone();
    let org_name = state::ME.read().as_ref().and_then(|m| m.organizations.iter().find(|o| o.id == org).map(|o| o.name.clone())).unwrap_or_default();
    let page_url = |id: &str| format!("/api/v1/organizations/{org}/scan/pages/{id}");

    let mobile_page = |id: &str| format!("/api/v1/organizations/{org}/scan/pages/{id}");
    let view_idx = viewing().and_then(|v| pages.iter().position(|p| *p == v));
    rsx! {
        // ---------------------------------------------------------------- Handy
        div { class: "space-y-4 p-4 md:hidden",
            div {
                h1 { class: "text-xl font-semibold tracking-tight", {tr("Scannen")} }
                p { class: "mt-1 text-sm text-muted-foreground",
                    if n == 0 { {tr("Jede Seite fotografieren – zusammen wird daraus ein PDF.")} } else { {tr("Weitere Seiten fotografieren oder fertig speichern.")} }
                }
            }
            if n == 0 {
                label { class: "flex cursor-pointer flex-col items-center justify-center gap-3 rounded-2xl bg-primary px-4 py-12 text-lg font-semibold text-primary-foreground shadow-lg active:scale-[0.99]",
                    if uploading() { icons::LoaderCircle { class: "size-10 animate-spin" } } else { icons::Camera { class: "size-10" } }
                    if uploading() { {tr("Wird hochgeladen …")} } else { {tr("Seite fotografieren")} }
                    input { id: "nrz-scan-camera", r#type: "file", accept: "image/*", capture: "environment", class: "hidden", disabled: uploading(), onchange: move |_| mobile_photos.call("nrz-scan-camera") }
                }
                p { class: "text-center text-xs text-muted-foreground", {tr("Tipp: Blatt auf einen dunklen Tisch legen, gerade von oben, gutes Licht.")} }
            } else {
                div { class: "grid grid-cols-3 gap-2",
                    for (i , p) in pages.iter().cloned().enumerate() {
                        button { key: "m-{p}", class: "relative overflow-hidden rounded-lg border bg-card",
                            onclick: { let p = p.clone(); move |_| viewing.set(Some(p.clone())) },
                            img { src: mobile_page(&p), alt: "", class: "aspect-[3/4] w-full object-cover" }
                            span { class: "absolute bottom-1 left-1 rounded bg-background/85 px-1.5 text-[11px] font-medium", "{i + 1}" }
                        }
                    }
                    label { class: "grid aspect-[3/4] cursor-pointer place-items-center rounded-lg border-2 border-dashed text-muted-foreground active:bg-accent",
                        span { class: "flex flex-col items-center gap-1 text-xs font-medium",
                            if uploading() { icons::LoaderCircle { class: "size-7 animate-spin" } } else { icons::Camera { class: "size-7" } }
                            {tr("Nächste Seite")}
                        }
                        input { id: "nrz-scan-next", r#type: "file", accept: "image/*", capture: "environment", class: "hidden", disabled: uploading(), onchange: move |_| mobile_photos.call("nrz-scan-next") }
                    }
                }
                button { class: "flex w-full items-center justify-center gap-2 rounded-2xl bg-primary px-4 py-4 text-base font-semibold text-primary-foreground shadow-lg disabled:opacity-60",
                    disabled: uploading(), onclick: move |_| finish.call(()),
                    icons::Check { class: "size-5" }
                    if n == 1 { {tr("Fertig – 1 Seite speichern")} } else { {trf("Fertig – {0} Seiten speichern", &[&n])} }
                }
                input { class: INPUT, placeholder: tr("Name (optional) – sonst schlägt die KI einen vor"), value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    onchange: move |e| update.call(json!({ "name": e.value() })),
                }
            }
            div { class: "grid grid-cols-2 gap-2",
                label { class: "{btn(BTN_OUTLINE)} h-12 cursor-pointer",
                    icons::Images { class: "size-4" } {tr("Aus Fotos")}
                    input { id: "nrz-scan-library", r#type: "file", accept: "image/*", multiple: true, class: "hidden", disabled: uploading(), onchange: move |_| mobile_photos.call("nrz-scan-library") }
                }
                label { class: "{btn(BTN_OUTLINE)} h-12 cursor-pointer",
                    icons::FileUp { class: "size-4" } {tr("PDF / Datei")}
                    input { r#type: "file", accept: "application/pdf,image/*", multiple: true, class: "hidden", disabled: uploading(), onchange: move |e| import_pdfs.call(e) }
                }
            }
            if status().ok {
                button { class: "{btn(BTN_OUTLINE)} h-12 w-full", disabled: scanning(), onclick: move |_| scan_now.call(()),
                    if scanning() { icons::LoaderCircle { class: "size-4 animate-spin" } {tr("Scanne …")} } else { icons::ScanLine { class: "size-4" } {tr("Mit dem Netzwerk-Scanner scannen")} }
                }
            }
            if n > 0 {
                button { class: "w-full py-2 text-sm text-muted-foreground", onclick: move |_| discard.call(()), {tr("Alle Seiten verwerfen")} }
            }
        }
        // Eine Seite gross: umsortieren oder entfernen
        if let (Some(v), Some(i)) = (viewing(), view_idx) {
            div { class: "fixed inset-0 z-[60] flex flex-col bg-black/95 pb-[env(safe-area-inset-bottom)] pt-[env(safe-area-inset-top)] md:hidden",
                div { class: "flex items-center justify-between px-4 py-3",
                    span { class: "text-sm font-medium", {trf("Seite {0} von {1}", &[&(i + 1), &(n)])} }
                    button { class: "{btn(BTN_GHOST)} {BTN_ICON}", title: tr("Schliessen"), onclick: move |_| viewing.set(None), icons::X { class: "size-5" } }
                }
                div { class: "grid min-h-0 flex-1 place-items-center p-3",
                    img { src: mobile_page(&v), alt: "", class: "max-h-full max-w-full rounded object-contain" }
                }
                div { class: "grid grid-cols-3 gap-2 p-4",
                    button { class: "{btn(BTN_OUTLINE)} h-12", disabled: i == 0, onclick: { let v = v.clone(); move |_| move_page.call((v.clone(), i.saturating_sub(1))) }, icons::ArrowLeft { class: "size-4" } {tr("Nach vorne")} }
                    button { class: "{btn(BTN_OUTLINE)} h-12", disabled: i + 1 >= n, onclick: { let v = v.clone(); move |_| move_page.call((v.clone(), i + 1)) }, {tr("Nach hinten")} icons::ArrowRight { class: "size-4" } }
                    button { class: "{btn(BTN_DANGER)} h-12", onclick: { let v = v.clone(); move |_| { remove_page.call(v.clone()); viewing.set(None); } }, icons::Trash2 { class: "size-4" } {tr("Löschen")} }
                }
            }
        }
        // ---------------------------------------------------------------- Desktop
        div { class: "hidden p-4 md:block md:p-6",
            PageHeader { title: tr("Scannen"), description: trf("Ablage in {0} – die KI schlägt danach Name, Tags und Metadaten vor, du gibst frei.", &[&(org_name)]),
                actions: rsx! {
                    Link { to: Route::Settings { org: org.clone(), section: "scanner".into() },
                        class: "inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs text-muted-foreground hover:text-foreground",
                        span { class: if status().ok { "size-2 rounded-full bg-success" } else { "size-2 rounded-full bg-muted-foreground/40" } }
                        if status().ok { {trf("Scanner bereit ({0})", &[&(status().state)])} } else if status().state.is_empty() { {tr("Scanner wird geprüft …")} } else { "{status().state}" }
                    }
                }
            }
            if !status().ok && status().state == {tr("Kein Scanner eingestellt")} {
                div { class: "mb-4", Callout { text: tr("Noch kein Scanner eingestellt – im Profilmenü unter Scanner eintragen oder im Netz suchen. Fotos und PDFs gehen auch ohne Scanner.") } }
            }

            div { class: "grid gap-5 xl:grid-cols-[minmax(0,1fr)_22rem]",
                section { class: "flex min-h-[32rem] flex-col overflow-hidden rounded-xl border bg-card",
                    if let Some(sel) = sel.clone() {
                        div { class: "relative grid flex-1 place-items-center overflow-auto bg-background p-5",
                            Loupe { src: page_url(&sel) }
                        }
                        div { class: "flex flex-wrap items-center justify-between gap-2 border-t px-4 py-2 text-sm",
                            span {
                                span { class: "font-medium", {trf("Seite {0} von {1}", &[&(idx + 1), &(n)])} }
                                span { class: "text-muted-foreground", {tr(" · Lupe: Maus über die Seite · Reihenfolge: unten ziehen")} }
                            }
                            span { class: "flex gap-1",
                                button { class: "{btn(BTN_GHOST)} {BTN_SM}", disabled: idx == 0, title: tr("Nach vorne"), onclick: { let s = sel.clone(); move |_| move_page.call((s.clone(), idx.saturating_sub(1))) }, icons::ArrowLeft { class: "size-4" } }
                                button { class: "{btn(BTN_GHOST)} {BTN_SM}", disabled: idx + 1 >= n, title: tr("Nach hinten"), onclick: { let s = sel.clone(); move |_| move_page.call((s.clone(), idx + 1)) }, icons::ArrowRight { class: "size-4" } }
                                button { class: "{btn(BTN_OUTLINE)} {BTN_SM}", onclick: { let s = sel.clone(); move |_| remove_page.call(s.clone()) }, icons::Trash2 { class: "size-4" } {tr("Seite entfernen")} }
                            }
                        }
                        div { class: "flex gap-3 overflow-x-auto border-t bg-card p-3",
                            ondragover: move |e| e.prevent_default(),
                            for (i , p) in pages.iter().cloned().enumerate() {
                                div {
                                    key: "{p}",
                                    draggable: "true",
                                    class: {
                                        let mut c = String::from("group relative w-24 shrink-0 cursor-grab rounded-lg border-2 p-1 transition ");
                                        c += if sel == p { "border-primary" } else { "border-transparent hover:border-border" };
                                        if drag_id().as_deref() == Some(p.as_str()) { c += " opacity-40"; }
                                        if drop_before().as_deref() == Some(p.as_str()) { c += " shadow-[-6px_0_0_-2px_var(--primary)]"; }
                                        c
                                    },
                                    onclick: { let p = p.clone(); move |_| selected.set(Some(p.clone())) },
                                    ondragstart: { let p = p.clone(); move |_| drag_id.set(Some(p.clone())) },
                                    ondragend: move |_| { drag_id.set(None); drop_before.set(None); },
                                    ondragover: { let p = p.clone(); move |e: DragEvent| { e.prevent_default(); if drag_id().as_deref() != Some(p.as_str()) { drop_before.set(Some(p.clone())); } } },
                                    ondrop: { let p = p.clone(); let pages = pages.clone(); move |e: DragEvent| {
                                        e.prevent_default();
                                        drop_before.set(None);
                                        if let Some(id) = drag_id.take() {
                                            if id != p {
                                                let target = pages.iter().filter(|x| **x != id).position(|x| *x == p).unwrap_or(0);
                                                move_page.call((id, target));
                                            }
                                        }
                                    } },
                                    img { src: page_url(&p), alt: "", class: "pointer-events-none aspect-[3/4] w-full rounded object-cover" }
                                    button { class: "absolute right-1.5 top-1.5 hidden size-5 place-items-center rounded-full bg-background/90 text-xs group-hover:grid",
                                        title: trf("Seite {0} entfernen", &[&(i + 1)]),
                                        onclick: { let p = p.clone(); move |e: MouseEvent| { e.stop_propagation(); remove_page.call(p.clone()); } },
                                        icons::X { class: "size-3" }
                                    }
                                    span { class: "mt-1 block text-center text-[11px] text-muted-foreground", {trf("Seite {0}", &[&(i + 1)])} }
                                }
                            }
                        }
                    } else {
                        div { class: "grid flex-1 place-items-center p-10 text-center",
                            div {
                                div { class: "mx-auto grid size-14 place-items-center rounded-2xl bg-muted text-muted-foreground", icons::ScanLine { class: "size-7" } }
                                p { class: "mt-4 text-lg font-medium", {tr("Noch keine Seiten")} }
                                p { class: "mt-2 text-sm text-muted-foreground", {tr("Blatt aufs Glas legen und auf „Scannen“ drücken.")} br {} {tr("Mehrere Seiten einfach nacheinander scannen – alles wird ein PDF.")} }
                            }
                        }
                    }
                }

                aside { class: "space-y-4",
                    div { class: "space-y-3 rounded-xl border bg-card p-4 shadow-sm",
                        button { class: "flex w-full items-center justify-center gap-3 rounded-xl bg-primary px-4 py-5 text-lg font-semibold text-primary-foreground shadow-xs hover:bg-primary/90 disabled:cursor-wait disabled:opacity-60",
                            disabled: scanning(), onclick: move |_| scan_now.call(()),
                            if scanning() {
                                icons::LoaderCircle { class: "size-6 animate-spin" } {tr("Scanne …")}
                            } else {
                                icons::ScanLine { class: "size-6" }
                                if n > 0 { {trf("Seite {0} scannen", &[&(n + 1)])} } else { {tr("Scannen")} }
                            }
                        }
                        Field { label: tr("Name (optional)"),
                            input { class: INPUT, placeholder: tr("leer lassen – die KI schlägt einen vor"), value: "{name}",
                                oninput: move |e| name.set(e.value()),
                                onchange: move |e| update.call(json!({ "name": e.value() })),
                            }
                        }
                        if n > 0 {
                            button { class: "{btn(BTN_PRIMARY)} w-full", onclick: move |_| finish.call(()),
                                icons::Check { class: "size-4" }
                                if n == 1 { {tr("1 Seite → zur Prüfung")} } else { {trf("{0} Seiten → zur Prüfung", &[&(n)])} }
                            }
                            button { class: "{btn(BTN_OUTLINE)} w-full", onclick: move |_| discard.call(()), icons::Trash2 { class: "size-4" } {tr("Alle Seiten verwerfen")} }
                        }
                    }
                    div { class: "rounded-xl border bg-card p-4 shadow-sm",
                        h2 { class: "mb-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground", {tr("Tags vorgeben")} }
                        if tags.is_empty() {
                            p { class: "text-sm text-muted-foreground", {tr("Noch keine Tags.")} " " Link { class: "underline", to: Route::Tags { org: org.clone() }, {tr("Tags anlegen")} } }
                        }
                        div { class: "flex flex-wrap gap-1.5",
                            for t in tags.iter().cloned() {
                                TagToggle { tag: t.clone(), on: hints.contains(&t.name),
                                    onclick: { let hints = hints.clone(); move |_| {
                                        let mut h = hints.clone();
                                        if let Some(i) = h.iter().position(|x| *x == t.name) { h.remove(i); } else { h.push(t.name.clone()); }
                                        update.call(json!({ "hintTags": h }));
                                    } }
                                }
                            }
                        }
                    }
                    details { class: "rounded-xl border bg-card p-4 shadow-sm",
                        summary { class: "cursor-pointer text-xs font-semibold uppercase tracking-wider text-muted-foreground", {tr("Scan-Einstellungen & Import")} }
                        div { class: "mt-3 grid gap-3",
                            div { class: "grid grid-cols-2 rounded-md border p-0.5 text-sm",
                                button { class: if color() { "rounded-sm bg-accent py-1.5" } else { "rounded-sm py-1.5 text-muted-foreground" }, onclick: move |_| { color.set(true); util::storage_set("nrz-scan-color", "true"); }, {tr("Farbe")} }
                                button { class: if !color() { "rounded-sm bg-accent py-1.5" } else { "rounded-sm py-1.5 text-muted-foreground" }, onclick: move |_| { color.set(false); util::storage_set("nrz-scan-color", "false"); }, {tr("Graustufen")} }
                            }
                            select { class: SELECT, onchange: move |e| { if let Ok(v) = e.value().parse() { dpi.set(v); util::storage_set("nrz-scan-dpi", &e.value()); } },
                                for d in [150i64, 300, 600] { option { value: "{d}", selected: dpi() == d, "{d} dpi" } }
                            }
                            label { class: "{btn(BTN_OUTLINE)} cursor-pointer", icons::ImageUp { class: "size-4" } {tr("Fotos als Seiten (JPG)")}
                                input { r#type: "file", accept: "image/jpeg", multiple: true, class: "hidden", onchange: add_photos }
                            }
                            label { class: "{btn(BTN_OUTLINE)} cursor-pointer", icons::FileUp { class: "size-4" } {tr("PDF prüfen lassen")}
                                input { r#type: "file", accept: "application/pdf", multiple: true, class: "hidden", onchange: move |e| import_pdfs.call(e) }
                            }
                        }
                    }
                }
            }
        }
    }
}

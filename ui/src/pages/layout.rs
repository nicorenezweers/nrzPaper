//! Rahmen: Anmeldung prüfen (AppLayout) und die Seitenleiste einer Organisation (OrgLayout)
//! mit Organisationswechsel, Navigation, Ordnerbaum und Profilmenü (dort die Einstellungen).

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::{Folder, Tag};
use crate::state::{self, ME, OVERVIEW, REFRESH};
use crate::Route;
use dioxus::prelude::*;

/// Lädt den angemeldeten Benutzer; ohne Anmeldung zur Anmeldeseite.
#[component]
pub fn AppLayout() -> Element {
    let nav = use_navigator();
    let me = use_resource(move || async move {
        let _ = REFRESH();
        state::load_me().await
    });
    match &*me.read() {
        None => rsx! { div { class: "grid h-screen place-items-center", Loading {} } },
        Some(None) => {
            let status = use_resource(|| api::get::<crate::models::AuthStatus>("/auth/status"));
            if let Some(Ok(s)) = &*status.read() {
                if s.needs_setup {
                    nav.replace(Route::Setup {});
                } else {
                    nav.replace(Route::Login { next: String::new() });
                }
            }
            rsx! { div { class: "grid h-screen place-items-center", Loading {} } }
        }
        Some(Some(_)) => rsx! { Outlet::<Route> {} },
    }
}

/// Seitenleiste und Kopf für alle Seiten einer Organisation.
#[component]
pub fn OrgLayout() -> Element {
    let route = use_route::<Route>();
    let org = route.org().unwrap_or_default().to_string();
    let mut mobile_open = use_signal(|| false);

    // Überblick (Tags, Ordner, Inbox-Zähler) laden, wenn sich die Organisation ändert oder etwas neu ist.
    let org_for_load = org.clone();
    let _overview = use_resource(use_reactive!(|org_for_load| async move {
        let _ = REFRESH();
        state::load_overview(&org_for_load).await;
    }));
    // Inbox-Zähler alle 20 s auffrischen (E-Mail-Abruf, KI im Hintergrund).
    let org_poll = org.clone();
    use_future(move || {
        let org = org_poll.clone();
        async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(20_000).await;
                state::load_overview(&org).await;
            }
        }
    });

    let Some(me) = ME.read().clone() else { return rsx! {} };
    if !me.organizations.iter().any(|o| o.id == org) {
        return rsx! {
            div { class: "grid h-screen place-items-center p-6",
                EmptyState { title: "Organisation nicht gefunden", description: "Es gibt sie nicht, oder du bist kein Mitglied.",
                    Link { class: btn(BTN_OUTLINE), to: Route::Home {}, "Zur Übersicht" }
                }
            }
        };
    }

    rsx! {
        div { class: "flex h-screen overflow-hidden",
            // Seitenleiste (Desktop)
            aside { class: "hidden w-64 shrink-0 border-r border-sidenav-border bg-sidenav md:flex md:flex-col",
                Sidebar { org: org.clone() }
            }
            // Seitenleiste (Handy) als Sheet
            if mobile_open() {
                div { class: "fixed inset-0 z-50 bg-black/60 md:hidden", onclick: move |_| mobile_open.set(false) }
                aside { class: "fixed inset-y-0 left-0 z-50 flex w-72 flex-col border-r bg-sidenav shadow-xl animate-in md:hidden",
                    onclick: move |_| mobile_open.set(false),
                    Sidebar { org: org.clone() }
                }
            }
            div { class: "flex min-w-0 flex-1 flex-col",
                TopBar { org: org.clone(), on_menu: move |_| mobile_open.set(true) }
                main { class: "min-h-0 flex-1 overflow-y-auto", Outlet::<Route> {} }
            }
        }
    }
}

#[component]
fn TopBar(org: String, on_menu: EventHandler<()>) -> Element {
    let nav = use_navigator();
    let route = use_route::<Route>();
    let current_q = match &route {
        Route::Documents { q, .. } | Route::FolderView { q, .. } => q.clone(),
        _ => String::new(),
    };
    let mut text = use_signal(|| current_q.clone());
    // Suche in der Adresse geändert (z. B. Tag-Filter) → Feld nachziehen
    use_effect(use_reactive!(|current_q| text.set(current_q)));
    let org2 = org.clone();
    let route2 = route.clone();
    let mut submit = move |value: String| {
        match &route2 {
            Route::FolderView { org, folder, .. } => nav.push(Route::FolderView { org: org.clone(), folder: folder.clone(), q: value }),
            _ => nav.push(Route::Documents { org: org2.clone(), q: value }),
        };
    };
    rsx! {
        header { class: "flex h-14 shrink-0 items-center gap-3 border-b px-4 md:px-6",
            button { class: "{btn(BTN_GHOST)} {BTN_ICON} md:hidden", onclick: move |_| on_menu.call(()), icons::Menu { class: "size-5" } }
            form { class: "relative w-full max-w-xl",
                onsubmit: move |e| {
                    e.prevent_default();
                    submit(text());
                },
                icons::Search { class: "pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" }
                input {
                    class: "{INPUT} pl-9",
                    r#type: "search",
                    placeholder: "Suchen … z. B. rechnung tag:steuern created:2026",
                    value: "{text}",
                    oninput: move |e| text.set(e.value()),
                }
            }
            SearchHelp {}
        }
    }
}

#[component]
fn SearchHelp() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        button { class: "{btn(BTN_GHOST)} {BTN_SM} hidden text-muted-foreground sm:inline-flex", onclick: move |_| open.set(true), icons::Info { class: "size-4" } "Suchhilfe" }
        Modal { open, title: "Suchhilfe", description: "Begriffe lassen sich frei kombinieren.",
            div { class: "grid gap-2 text-sm",
                for (code , text) in [
                    ("rechnung zürich", "beide Wörter, Anfang genügt, Umlaute egal"),
                    ("\"mobile abo\"", "genau diese Wortfolge"),
                    ("name: / content:", "nur im Namen bzw. im erkannten Text"),
                    ("tag:steuern  -tag:privat", "mit bzw. ohne Tag"),
                    ("has:tags", "nur Dokumente mit Tags"),
                    ("type:pdf / image / text", "Dateityp"),
                    ("created:2026  created:>2026-06-01", "Hochgeladen im Jahr / ab Datum"),
                    ("status:failed", "Texterkennung fehlgeschlagen"),
                ] {
                    div { class: "flex flex-wrap items-baseline gap-x-3 gap-y-1",
                        code { class: "rounded bg-muted px-1.5 py-0.5 font-mono text-xs", "{code}" }
                        span { class: "text-muted-foreground", "{text}" }
                    }
                }
            }
        }
    }
}

#[component]
fn NavLink(to: Route, #[props(into)] label: String, icon: Element, active: bool, #[props(default)] badge: i64) -> Element {
    let c = if active {
        "bg-sidenav-accent font-medium text-sidenav-accent-foreground"
    } else {
        "text-muted-foreground hover:bg-sidenav-accent/60 hover:text-sidenav-accent-foreground"
    };
    rsx! {
        Link { to, class: "flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-sm transition-colors [&_svg]:size-4 [&_svg]:shrink-0 {c}",
            {icon}
            span { class: "flex-1 truncate", "{label}" }
            if badge > 0 {
                span { class: "rounded-full bg-primary px-1.5 text-[11px] font-semibold leading-5 text-primary-foreground", "{badge}" }
            }
        }
    }
}

#[component]
fn Sidebar(org: String) -> Element {
    let nav = use_navigator();
    let route = use_route::<Route>();
    let me = ME.read().clone().unwrap_or_default();
    let overview = OVERVIEW.read().clone().filter(|(o, _)| *o == org).map(|(_, v)| v).unwrap_or_default();
    let current = me.organizations.iter().find(|o| o.id == org).cloned().unwrap_or_default();
    let initial = current.name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    let mut folder_dialog = use_signal(|| false);
    let mut editing = use_signal(|| None::<Folder>);
    let active_folder = match &route {
        Route::FolderView { folder, .. } => folder.clone(),
        _ => String::new(),
    };

    let is = |r: &Route| std::mem::discriminant(r) == std::mem::discriminant(&route);
    rsx! {
        // Organisation wechseln
        div { class: "p-3",
            Dropdown {
                trigger_class: "flex w-full items-center gap-2.5 rounded-lg border bg-card px-2.5 py-2 text-left hover:bg-accent/50",
                class: "w-[14.5rem]",
                trigger: rsx! {
                    span { class: "grid size-8 place-items-center rounded-md bg-primary text-sm font-semibold text-primary-foreground", "{initial}" }
                    span { class: "min-w-0 flex-1",
                        span { class: "block truncate text-sm font-semibold", "{current.name}" }
                        span { class: "block text-xs text-muted-foreground", "{crate::models::role_label(&current.role)}" }
                    }
                    icons::ChevronsUpDown { class: "size-4 text-muted-foreground" }
                },
                MenuLabel { "Organisationen" }
                for o in me.organizations.iter().cloned() {
                    MenuItem { active: o.id == org, onclick: move |_| { nav.push(Route::Documents { org: o.id.clone(), q: String::new() }); },
                        icons::Building2 {}
                        span { class: "truncate", "{o.name}" }
                    }
                }
                MenuSeparator {}
                MenuItem { onclick: move |_| { nav.push(Route::NewOrg {}); }, icons::Plus {} "Neue Organisation" }
            }
        }

        nav { class: "flex flex-col gap-0.5 px-3",
            NavLink { to: Route::Documents { org: org.clone(), q: String::new() }, label: "Dokumente", active: is(&Route::Documents { org: String::new(), q: String::new() }) || is(&Route::DocumentPage { org: String::new(), id: String::new() }), icon: rsx! { icons::FileText {} } }
            NavLink { to: Route::Inbox { org: org.clone(), item: String::new() }, label: "Inbox", badge: overview.review_count, active: is(&Route::Inbox { org: String::new(), item: String::new() }), icon: rsx! { icons::Inbox {} } }
            NavLink { to: Route::Scan { org: org.clone() }, label: "Scannen", active: is(&Route::Scan { org: String::new() }), icon: rsx! { icons::ScanLine {} } }
            NavLink { to: Route::Tags { org: org.clone() }, label: "Tags", active: is(&Route::Tags { org: String::new() }), icon: rsx! { icons::Tag {} } }
            NavLink { to: Route::Trash { org: org.clone() }, label: "Papierkorb", active: is(&Route::Trash { org: String::new() }), icon: rsx! { icons::Trash2 {} } }
        }

        // Ordner
        div { class: "mt-5 flex items-center justify-between px-5",
            span { class: "text-xs font-medium uppercase tracking-wider text-muted-foreground", "Ordner" }
            button { class: "grid size-6 place-items-center rounded-md text-muted-foreground hover:bg-sidenav-accent hover:text-foreground", title: "Neuer Ordner",
                onclick: move |_| { editing.set(None); folder_dialog.set(true); },
                icons::Plus { class: "size-4" }
            }
        }
        div { class: "mt-1 min-h-0 flex-1 overflow-y-auto px-3 pb-3",
            if overview.folders.is_empty() {
                button { class: "w-full rounded-md px-2.5 py-1.5 text-left text-sm text-muted-foreground hover:bg-sidenav-accent/60",
                    onclick: move |_| { editing.set(None); folder_dialog.set(true); },
                    "Ordner aus Tags anlegen …"
                }
            }
            for f in overview.folders.iter().cloned() {
                FolderRow { key: "{f.id}", org: org.clone(), folder: f.clone(), active: f.id == active_folder,
                    on_edit: move |f: Folder| { editing.set(Some(f)); folder_dialog.set(true); },
                }
            }
        }

        // Profil mit Einstellungen
        div { class: "border-t border-sidenav-border p-3",
            ProfileMenu { org: org.clone(), is_admin: me.user.is_admin, username: me.user.username.clone(), role: overview.role.clone() }
        }

        FolderDialog { open: folder_dialog, org: org.clone(), folder: editing(), folders: overview.folders.clone(), tags: overview.tags.clone() }
    }
}

#[component]
fn FolderRow(org: String, folder: Folder, active: bool, on_edit: EventHandler<Folder>) -> Element {
    let c = if active {
        "bg-sidenav-accent font-medium text-sidenav-accent-foreground"
    } else {
        "text-muted-foreground hover:bg-sidenav-accent/60 hover:text-sidenav-accent-foreground"
    };
    let pad = 10 + folder.depth * 14;
    let f2 = folder.clone();
    let org2 = org.clone();
    rsx! {
        div { class: "group relative",
            Link { to: Route::FolderView { org: org.clone(), folder: folder.id.clone(), q: String::new() },
                class: "flex items-center gap-2 rounded-md py-1.5 pr-8 text-sm {c}",
                style: "padding-left: {pad}px",
                if active { icons::FolderOpen { class: "size-4 shrink-0" } } else { icons::Folder { class: "size-4 shrink-0" } }
                span { class: "min-w-0 flex-1 truncate", "{folder.name}" }
                span { class: "text-[11px] tabular-nums text-muted-foreground group-hover:invisible", "{folder.count}" }
            }
            div { class: "absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100",
                Dropdown { align: Align::End,
                    trigger_class: "grid size-6 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground",
                    trigger: rsx! { icons::Ellipsis { class: "size-4" } },
                    MenuItem { onclick: move |_| on_edit.call(f2.clone()), icons::Pencil {} "Bearbeiten" }
                    MenuItem { destructive: true,
                        onclick: move |_| {
                            let (org, f) = (org2.clone(), folder.clone());
                            spawn(async move {
                                let ask = Ask::new(format!("Ordner „{}“ löschen?", f.name), "Unterordner werden mitgelöscht. Dokumente und Tags bleiben.").confirm("Löschen").destructive();
                                if confirm(ask).await {
                                    match api::delete(&format!("/organizations/{org}/folders/{}", f.id)).await {
                                        Ok(()) => { toast("Ordner gelöscht."); state::refresh(); }
                                        Err(e) => toast_error(e),
                                    }
                                }
                            });
                        },
                        icons::Trash2 {} "Löschen"
                    }
                }
            }
        }
    }
}

/// Ordner anlegen oder bearbeiten: Name, übergeordneter Ordner, Tags.
#[component]
pub fn FolderDialog(open: Signal<bool>, org: String, folder: Option<Folder>, folders: Vec<Folder>, tags: Vec<Tag>) -> Element {
    let mut name = use_signal(String::new);
    let mut parent = use_signal(String::new);
    let mut chosen = use_signal(Vec::<String>::new);
    let mut busy = use_signal(|| false);
    let editing_id = folder.as_ref().map(|f| f.id.clone()).unwrap_or_default();
    let init = folder.clone();
    // Beim Öffnen die Werte setzen
    use_effect(use_reactive!(|init| {
        if open() {
            name.set(init.as_ref().map(|f| f.name.clone()).unwrap_or_default());
            parent.set(init.as_ref().and_then(|f| f.parent_id.clone()).unwrap_or_default());
            chosen.set(init.as_ref().map(|f| f.tag_ids.clone()).unwrap_or_default());
        }
    }));
    let title = if folder.is_some() { "Ordner bearbeiten" } else { "Neuer Ordner" };
    let eid = editing_id.clone();
    let save = move |_| {
        let (org, eid) = (org.clone(), eid.clone());
        spawn(async move {
            busy.set(true);
            let body = serde_json::json!({ "name": name(), "parentId": parent(), "tagIds": chosen() });
            let res: api::ApiResult<serde_json::Value> = if eid.is_empty() {
                api::post(&format!("/organizations/{org}/folders"), &body).await
            } else {
                api::put(&format!("/organizations/{org}/folders/{eid}"), &body).await
            };
            busy.set(false);
            match res {
                Ok(_) => {
                    toast("Ordner gespeichert.");
                    open.set(false);
                    state::refresh();
                }
                Err(e) => toast_error(e),
            }
        });
    };
    rsx! {
        Modal { open, title, description: "Ein Ordner zeigt alle Dokumente mit mindestens einem seiner Tags – Unterordner eingeschlossen.",
            div { class: "grid gap-4",
                Field { label: "Name",
                    input { class: INPUT, value: "{name}", placeholder: "z. B. Finanzen", oninput: move |e| name.set(e.value()) }
                }
                Field { label: "Liegt in",
                    select { class: SELECT, value: "{parent}", onchange: move |e| parent.set(e.value()),
                        option { value: "", "— oberste Ebene —" }
                        for f in folders.iter().filter(|f| f.id != editing_id) {
                            option { value: "{f.id}", selected: f.id == parent(), {format!("{}{}", "  · ".repeat(f.depth), f.name)} }
                        }
                    }
                }
                div { class: "grid gap-2",
                    span { class: "text-sm font-medium", "Tags in diesem Ordner" }
                    if tags.is_empty() {
                        p { class: "text-sm text-muted-foreground", "Noch keine Tags – zuerst unter „Tags“ anlegen." }
                    }
                    div { class: "flex flex-wrap gap-1.5",
                        for t in tags.iter().cloned() {
                            TagToggle { tag: t.clone(), on: chosen().contains(&t.id),
                                onclick: move |_| {
                                    let id = t.id.clone();
                                    let mut c = chosen.write();
                                    if let Some(i) = c.iter().position(|x| *x == id) { c.remove(i); } else { c.push(id); }
                                }
                            }
                        }
                    }
                }
                div { class: "flex justify-end gap-2 pt-2",
                    button { class: btn(BTN_OUTLINE), onclick: move |_| open.set(false), "Abbrechen" }
                    button { class: btn(BTN_PRIMARY), disabled: busy() || name().trim().is_empty(), onclick: save, "Speichern" }
                }
            }
        }
    }
}

#[component]
fn ProfileMenu(org: String, is_admin: bool, username: String, role: String) -> Element {
    let nav = use_navigator();
    let initial = username.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    let go = move |section: &'static str| {
        let org = org.clone();
        move |_| { nav.push(Route::Settings { org: org.clone(), section: section.into() }); }
    };
    let can_manage = matches!(role.as_str(), "owner" | "admin");
    rsx! {
        Dropdown { side: Side::Top, class: "w-[14.5rem]",
            trigger_class: "flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left hover:bg-sidenav-accent",
            trigger: rsx! {
                span { class: "grid size-8 place-items-center rounded-full bg-secondary text-sm font-semibold", "{initial}" }
                span { class: "min-w-0 flex-1",
                    span { class: "block truncate text-sm font-medium", "{username}" }
                    span { class: "block text-xs text-muted-foreground", if is_admin { "Administrator" } else { "Konto" } }
                }
                icons::ChevronsUpDown { class: "size-4 text-muted-foreground" }
            },
            MenuLabel { "Konto" }
            MenuItem { onclick: go("profile"), icons::User {} "Profil & Passwort" }
            MenuItem { onclick: go("api-keys"), icons::Key {} "API-Keys" }
            MenuItem { onclick: go("scanner"), icons::Printer {} "Scanner" }
            MenuItem { onclick: go("mail"), icons::Mail {} "E-Mail-Eingang" }
            MenuSeparator {}
            MenuLabel { "Organisation" }
            MenuItem { onclick: go("organization"), icons::Building2 {} "Allgemein & Mitglieder" }
            MenuItem { onclick: go("tagging-rules"), icons::Wand {} "Tagging-Regeln" }
            MenuItem { onclick: go("properties"), icons::SlidersHorizontal {} "Eigenschaften" }
            if can_manage {
                MenuItem { onclick: go("webhooks"), icons::Webhook {} "Webhooks" }
            }
            if is_admin {
                MenuSeparator {}
                MenuLabel { "Instanz" }
                MenuItem { onclick: go("ai"), icons::Sparkles {} "KI (OpenRouter)" }
                MenuItem { onclick: go("instance"), icons::ShieldCheck {} "Registrierung & Benutzer" }
            }
            MenuSeparator {}
            MenuItem {
                onclick: move |_| {
                    spawn(async move {
                        let _: api::ApiResult<serde_json::Value> = api::action("/auth/logout").await;
                        *ME.write() = None;
                        nav.replace(Route::Login { next: String::new() });
                    });
                },
                icons::LogOut {}
                "Abmelden"
            }
        }
    }
}

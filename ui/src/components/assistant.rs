//! Dieter, der Assistent: Chat unten rechts. Fragen zu den eigenen Dokumenten („Wie lautet
//! meine Versicherungsnummer?“, „Alle Lohnabrechnungen von X 2025“) – Dieter sucht, liest und
//! antwortet mit Links zu den Dokumenten, auf die er sich stützt.

use crate::api;
use crate::i18n::{tr, trf};
use crate::state::OVERVIEW;
use crate::util;
use crate::Route;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Source {
    id: String,
    name: String,
    created_at: i64,
    tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Msg {
    role: String,
    content: String,
    sources: Vec<Source>,
    error: bool,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Answer {
    answer: String,
    sources: Vec<Source>,
    considered: usize,
}

/// Gespräche je Organisation – bleiben beim Seitenwechsel erhalten.
static CHATS: GlobalSignal<HashMap<String, Vec<Msg>>> = Signal::global(HashMap::new);
static OPEN: GlobalSignal<bool> = Signal::global(|| false);

/// Sehr kleines Markdown: **fett**, Listen mit -/*/1., Zeilen.
fn markdown(text: &str) -> Element {
    let inline = |line: &str| -> Element {
        let parts: Vec<&str> = line.split("**").collect();
        rsx! {
            for (i , p) in parts.iter().enumerate() {
                if i % 2 == 1 { strong { "{p}" } } else { "{p}" }
            }
        }
    };
    let mut blocks: Vec<Element> = Vec::new();
    let mut list: Vec<String> = Vec::new();
    let flush = |list: &mut Vec<String>, blocks: &mut Vec<Element>| {
        if !list.is_empty() {
            let items = std::mem::take(list);
            blocks.push(rsx! { ul { class: "my-1 list-disc space-y-0.5 pl-5", for it in items { li { {inline(&it)} } } } });
        }
    };
    for raw in text.lines() {
        let line = raw.trim();
        let item = line
            .strip_prefix("- ")
            .or_else(|| line.strip_prefix("* "))
            .or_else(|| line.strip_prefix("• "))
            .or_else(|| {
                let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
                (digits > 0 && line[digits..].starts_with(". ")).then(|| &line[digits + 2..])
            });
        match item {
            Some(it) => list.push(it.to_string()),
            None => {
                flush(&mut list, &mut blocks);
                if !line.is_empty() {
                    let l = line.trim_start_matches('#').trim().to_string();
                    blocks.push(rsx! { p { class: "my-1", {inline(&l)} } });
                }
            }
        }
    }
    flush(&mut list, &mut blocks);
    rsx! { for b in blocks { {b} } }
}

#[component]
pub fn Assistant(org: String) -> Element {
    let mut input = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let available = OVERVIEW.read().as_ref().filter(|(o, _)| *o == org).is_some_and(|(_, v)| v.ai_available);
    let messages = CHATS.read().get(&org).cloned().unwrap_or_default();

    let o = org.clone();
    let send = use_callback(move |text: String| {
        let text = text.trim().to_string();
        if text.is_empty() || busy() {
            return;
        }
        let org = o.clone();
        CHATS.write().entry(org.clone()).or_default().push(Msg { role: "user".into(), content: text, ..Default::default() });
        input.set(String::new());
        busy.set(true);
        spawn(async move {
            let history: Vec<serde_json::Value> = CHATS
                .read()
                .get(&org)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|m| !m.error)
                .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
                .collect();
            let res = api::post::<Answer, _>(&format!("/organizations/{org}/assistant"), &serde_json::json!({ "messages": history })).await;
            let msg = match res {
                Ok(a) => Msg { role: "assistant".into(), content: a.answer, sources: a.sources, error: false },
                Err(e) => Msg { role: "assistant".into(), content: e.message, sources: Vec::new(), error: true },
            };
            CHATS.write().entry(org).or_default().push(msg);
            busy.set(false);
            // ans Ende scrollen
            document::eval("requestAnimationFrame(() => { const c = document.getElementById('dieter-log'); if (c) c.scrollTop = c.scrollHeight; })");
        });
    });
    let suggestions = [
        tr("Wie lautet meine Versicherungsnummer?"),
        tr("Zeig mir alle Lohnabrechnungen von 2026."),
        tr("Welche Rechnungen sind noch offen?"),
    ];
    let org_reset = org.clone();

    rsx! {
        if !OPEN() {
            button {
                class: "fixed bottom-5 right-5 z-40 flex items-center gap-2 rounded-full border bg-primary px-4 py-3 text-sm font-medium text-primary-foreground shadow-lg transition hover:scale-[1.03] animate-in",
                title: tr("Dieter fragen"),
                onclick: move |_| {
                    *OPEN.write() = true;
                    document::eval("setTimeout(() => document.getElementById('dieter-input')?.focus(), 50)");
                },
                icons::MessageCircle { class: "size-5" }
                span { class: "hidden sm:inline", {tr("Dieter fragen")} }
            }
        } else {
            div { class: "fixed inset-x-2 bottom-2 z-40 flex h-[min(36rem,85dvh)] flex-col overflow-hidden rounded-2xl border bg-popover text-popover-foreground shadow-2xl animate-toast-in sm:inset-x-auto sm:right-5 sm:bottom-5 sm:w-[25rem]",
                role: "dialog",
                onkeydown: move |e| if e.key() == Key::Escape { *OPEN.write() = false },
                // Kopf
                div { class: "flex items-center gap-3 border-b px-4 py-3",
                    span { class: "grid size-9 place-items-center rounded-full bg-primary text-sm font-bold text-primary-foreground", "D" }
                    div { class: "min-w-0 flex-1",
                        p { class: "font-semibold leading-tight", "Dieter" }
                        p { class: "text-xs text-muted-foreground", {tr("Dein Assistent für die Dokumente")} }
                    }
                    if !messages.is_empty() {
                        button { class: "grid size-8 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground", title: tr("Neues Gespräch"),
                            onclick: move |_| { CHATS.write().remove(&org_reset); },
                            icons::SquarePen { class: "size-4" }
                        }
                    }
                    button { class: "grid size-8 place-items-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground", title: tr("Schliessen"),
                        onclick: move |_| *OPEN.write() = false,
                        icons::X { class: "size-4" }
                    }
                }
                // Verlauf
                div { id: "dieter-log", class: "min-h-0 flex-1 space-y-3 overflow-y-auto px-4 py-4 text-sm",
                    if messages.is_empty() {
                        div { class: "rounded-xl bg-accent/50 p-3",
                            p { {tr("Hallo, ich bin Dieter. Frag mich nach deinen Dokumenten – ich suche, lese und zeige dir die Quellen.")} }
                        }
                        if available {
                            div { class: "grid gap-1.5",
                                for s in suggestions.iter().cloned() {
                                    button { class: "rounded-lg border px-3 py-2 text-left text-xs text-muted-foreground hover:bg-accent hover:text-foreground",
                                        onclick: move |_| send.call(s.clone()), "{s}" }
                                }
                            }
                        } else {
                            p { class: "rounded-lg border border-dashed p-3 text-xs text-muted-foreground", {tr("Für Dieter braucht es eine KI – ein Administrator richtet sie unter Einstellungen → KI & Modelle ein.")} }
                        }
                    }
                    for (i , m) in messages.iter().enumerate() {
                        if m.role == "user" {
                            div { key: "{i}", class: "ml-8 rounded-2xl rounded-br-md bg-primary px-3 py-2 text-primary-foreground whitespace-pre-wrap", "{m.content}" }
                        } else {
                            div { key: "{i}", class: "mr-4 grid gap-2",
                                div { class: if m.error { "rounded-2xl rounded-bl-md border border-destructive/50 bg-destructive/10 px-3 py-2" } else { "rounded-2xl rounded-bl-md bg-accent px-3 py-2" },
                                    {markdown(&m.content)}
                                }
                                if !m.sources.is_empty() {
                                    div { class: "grid gap-1",
                                        p { class: "text-[11px] uppercase tracking-wider text-muted-foreground",
                                            if m.sources.len() == 1 { {tr("Quelle")} } else { {trf("{0} Quellen", &[&m.sources.len()])} }
                                        }
                                        for s in m.sources.iter().cloned() {
                                            Link { key: "{s.id}", to: Route::DocumentPage { org: org.clone(), id: s.id.clone() },
                                                class: "flex items-center gap-2 rounded-lg border bg-card px-2.5 py-1.5 text-xs hover:bg-accent",
                                                icons::FileText { class: "size-3.5 shrink-0 text-muted-foreground" }
                                                span { class: "min-w-0 flex-1 truncate", "{s.name}" }
                                                span { class: "shrink-0 text-muted-foreground", "{util::date(s.created_at)}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if busy() {
                        div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                            icons::LoaderCircle { class: "size-4 animate-spin" }
                            {tr("Dieter sucht in deinen Dokumenten …")}
                        }
                    }
                }
                // Eingabe
                form { class: "flex items-end gap-2 border-t p-3",
                    onsubmit: move |e| { e.prevent_default(); send.call(input()); },
                    textarea {
                        id: "dieter-input",
                        class: "max-h-32 min-h-10 flex-1 resize-none rounded-lg border border-input bg-transparent px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30",
                        rows: 1,
                        disabled: !available,
                        placeholder: tr("Frag Dieter …"),
                        value: "{input}",
                        oninput: move |e| input.set(e.value()),
                        onkeydown: move |e| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); send.call(input()); },
                    }
                    button { class: "grid size-10 shrink-0 place-items-center rounded-lg bg-primary text-primary-foreground disabled:opacity-50",
                        r#type: "submit", disabled: busy() || input().trim().is_empty() || !available, title: tr("Senden"),
                        icons::SendHorizontal { class: "size-4" }
                    }
                }
            }
        }
    }
}

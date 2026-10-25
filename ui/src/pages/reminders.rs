//! Alle eigenen offenen Erinnerungen der Organisation: fällige oben, dann die nächsten.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::i18n::{tr, trf};
use crate::models::*;
use crate::state::{self, REFRESH};
use crate::util;
use crate::Route;
use dioxus::prelude::*;
use serde_json::json;

#[component]
pub fn RemindersPage(org: String) -> Element {
    let mut version = use_signal(|| 0u64);
    let o = org.clone();
    let list = use_resource(use_reactive!(|o| async move {
        let _ = (version(), REFRESH());
        api::get::<Reminders>(&format!("/organizations/{o}/reminders")).await.map(|r| r.reminders)
    }));
    let now = util::now();
    let act = move |id: String, body: serde_json::Value, msg: String| {
        spawn(async move {
            match api::put::<serde_json::Value, _>(&format!("/reminders/{id}"), &body).await {
                Ok(_) => { toast(msg); version += 1; state::refresh(); }
                Err(e) => toast_error(e),
            }
        });
    };
    rsx! {
        div { class: "mx-auto max-w-3xl p-4 md:p-6",
            PageHeader { title: tr("Erinnerungen"), description: tr("Deine offenen Erinnerungen in dieser Organisation. Setzen lassen sie sich auf jeder Dokumentseite.") }
            match &*list.read() {
                None => rsx! { Loading {} },
                Some(Err(e)) => rsx! { Callout { text: e.message.clone(), error: true } },
                Some(Ok(l)) if l.is_empty() => rsx! {
                    EmptyState { title: tr("Keine offenen Erinnerungen"), description: tr("Auf einer Dokumentseite unter „Erinnerungen“ z. B. „in 10 Tagen“ wählen."), icon: rsx! { icons::BellOff {} } }
                },
                Some(Ok(l)) => rsx! {
                    div { class: "grid gap-2",
                        for r in l.iter().cloned() {
                            div { key: "{r.id}", class: if r.due_at <= now { "flex flex-wrap items-center gap-3 rounded-xl border border-destructive/50 bg-destructive/10 px-4 py-3" } else { "flex flex-wrap items-center gap-3 rounded-xl border bg-card px-4 py-3" },
                                if r.due_at <= now { icons::BellRing { class: "size-5 text-destructive" } } else { icons::Bell { class: "size-5 text-muted-foreground" } }
                                div { class: "min-w-0 flex-1",
                                    Link { class: "block truncate font-medium hover:underline", to: Route::DocumentPage { org: org.clone(), id: r.document_id.clone() }, "{r.document_name}" }
                                    p { class: "text-sm text-muted-foreground",
                                        if r.due_at <= now { {trf("Fällig seit {0}", &[&util::date(r.due_at)])} } else { {trf("Am {0}", &[&util::date(r.due_at)])} }
                                        if !r.note.is_empty() { " · {r.note}" }
                                    }
                                }
                                div { class: "flex gap-1",
                                    button { class: "{btn(BTN_GHOST)} {BTN_SM}", onclick: { let id = r.id.clone(); move |_| act(id.clone(), json!({ "snoozeDays": 1 }), tr("Auf morgen verschoben.")) },
                                        icons::AlarmClock { class: "size-4" } {tr("Morgen")}
                                    }
                                    button { class: "{btn(BTN_GHOST)} {BTN_SM}", onclick: { let id = r.id.clone(); move |_| act(id.clone(), json!({ "snoozeDays": 7 }), tr("Um eine Woche verschoben.")) },
                                        {tr("+1 Woche")}
                                    }
                                    button { class: "{btn(BTN_OUTLINE)} {BTN_SM}", onclick: { let id = r.id.clone(); move |_| act(id.clone(), json!({ "done": true }), tr("Erinnerung erledigt.")) },
                                        icons::Check { class: "size-4" } {tr("Erledigt")}
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

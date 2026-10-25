//! Papierkorb: wiederherstellen, endgültig löschen, leeren – alles mit eigenen Rückfragen.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::DocumentList;
use crate::state::{self, OVERVIEW, REFRESH};
use crate::util;
use crate::Route;
use dioxus::prelude::*;
use crate::i18n::{tr, trf};

#[component]
pub fn Trash(org: String) -> Element {
    let mut version = use_signal(|| 0u64);
    let o = org.clone();
    let list = use_resource(use_reactive!(|o| async move {
        let _ = (version(), REFRESH());
        api::get::<DocumentList>(&format!("/organizations/{o}/documents/deleted?pageSize=200")).await
    }));
    let role = OVERVIEW.read().clone().filter(|(x, _)| *x == org).map(|(_, v)| v.role).unwrap_or_default();
    let can_empty = matches!(role.as_str(), "owner" | "admin");
    let docs = list.read().clone().and_then(|r| r.ok()).map(|l| l.documents).unwrap_or_default();
    let n = docs.len();
    let org_empty = org.clone();
    rsx! {
        div { class: "mx-auto max-w-5xl p-4 md:p-6",
            PageHeader { title: tr("Papierkorb"), description: tr("Gelöschte Dokumente bleiben 30 Tage, dann werden sie endgültig entfernt."),
                actions: rsx! {
                    if can_empty && n > 0 {
                        button { class: btn(BTN_DANGER),
                            onclick: move |_| {
                                let org = org_empty.clone();
                                spawn(async move {
                                    let ask = Ask::new(tr("Papierkorb leeren?"), trf("Alle {0} Dokumente werden endgültig gelöscht. Das lässt sich nicht rückgängig machen.", &[&(n)])).confirm(tr("Endgültig löschen")).destructive();
                                    if confirm(ask).await {
                                        match api::action::<serde_json::Value>(&format!("/organizations/{org}/trash/empty")).await {
                                            Ok(_) => { toast(tr("Papierkorb geleert.")); version += 1; state::refresh(); }
                                            Err(e) => toast_error(e),
                                        }
                                    }
                                });
                            },
                            icons::Trash2 { class: "size-4" } {tr("Papierkorb leeren")}
                        }
                    }
                }
            }
            match &*list.read() {
                None => rsx! { Loading {} },
                Some(Err(e)) => rsx! { Callout { text: e.message.clone(), error: true } },
                Some(Ok(_)) if n == 0 => rsx! { EmptyState { title: {tr("Der Papierkorb ist leer")}, icon: rsx! { icons::Trash2 {} } } },
                Some(Ok(_)) => rsx! {
                    div { class: "divide-y overflow-hidden rounded-xl border bg-card",
                        for d in docs.iter().cloned() {
                            div { key: "{d.id}", class: "flex flex-wrap items-center gap-3 px-4 py-3",
                                span { class: "grid size-9 shrink-0 place-items-center rounded-md bg-muted text-[10px] font-semibold text-muted-foreground", "{d.type_label()}" }
                                div { class: "min-w-0 flex-1",
                                    Link { to: Route::DocumentPage { org: org.clone(), id: d.id.clone() }, class: "block truncate font-medium hover:underline", "{d.name}" }
                                    p { class: "text-xs text-muted-foreground",
                                        {trf("Gelöscht {0}", &[&(util::datetime(d.deleted_at.unwrap_or_default()))])}
                                        {
                                            let left = 30 - (util::now() - d.deleted_at.unwrap_or_default()) / 86_400;
                                            rsx! { {trf(" · noch {0} Tage", &[&(left.max(0))])} }
                                        }
                                    }
                                }
                                button { class: "{btn(BTN_OUTLINE)} {BTN_SM}",
                                    onclick: { let (org, id) = (org.clone(), d.id.clone()); move |_| {
                                        let (org, id) = (org.clone(), id.clone());
                                        spawn(async move {
                                            match api::action::<serde_json::Value>(&format!("/organizations/{org}/documents/{id}/restore")).await {
                                                Ok(_) => { toast({tr("Wiederhergestellt.")}); version += 1; state::refresh(); }
                                                Err(e) => toast_error(e),
                                            }
                                        });
                                    } },
                                    icons::Undo2 { class: "size-4" } {tr("Wiederherstellen")}
                                }
                                button { class: "{btn(BTN_GHOST)} {BTN_SM} text-muted-foreground hover:text-destructive",
                                    onclick: { let (org, d) = (org.clone(), d.clone()); move |_| {
                                        let (org, d) = (org.clone(), d.clone());
                                        spawn(async move {
                                            let ask = Ask::new({tr("Endgültig löschen?")}, {trf("„{0}“ wird unwiderruflich gelöscht.", &[&(d.name)])}).confirm({tr("Endgültig löschen")}).destructive();
                                            if confirm(ask).await {
                                                match api::delete(&format!("/organizations/{org}/documents/{}/permanent", d.id)).await {
                                                    Ok(()) => { toast({tr("Endgültig gelöscht.")}); version += 1; }
                                                    Err(e) => toast_error(e),
                                                }
                                            }
                                        });
                                    } },
                                    {tr("Endgültig löschen")}
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

//! Offene Einladungen in andere Organisationen: in der Seitenleiste über dem Profil und auf
//! der Seite „Neue Organisation“ (wer noch keine hat). Annehmen öffnet die Organisation.

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::models::role_label;
use crate::state::{self, INVITATIONS};
use crate::Route;
use dioxus::prelude::*;
use crate::i18n::{tr, trf};

#[component]
pub fn Invitations(#[props(default)] large: bool) -> Element {
    let nav = use_navigator();
    let list = INVITATIONS.read().clone();
    if list.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: if large { "grid gap-2" } else { "grid gap-2 px-3 pb-2" },
            for inv in list {
                div { key: "{inv.id}", class: "rounded-lg border bg-card p-3 text-sm shadow-sm animate-in",
                    p { class: "flex items-center gap-2 font-medium",
                        icons::MailOpen { class: "size-4 shrink-0 text-muted-foreground" }
                        span { class: "truncate", {trf("Einladung: {0}", &[&(inv.organization_name)])} }
                    }
                    p { class: "mt-0.5 text-xs text-muted-foreground",
                        {trf("als {0}", &[&role_label(&inv.role)])}
                        if let Some(by) = inv.invited_by.as_ref() { " · " {trf("von {0}", &[by])} }
                    }
                    div { class: "mt-2 flex gap-1.5",
                        button { class: "{btn(BTN_PRIMARY)} {BTN_SM} flex-1",
                            onclick: { let id = inv.id.clone(); move |_| {
                                let id = id.clone();
                                spawn(async move {
                                    match api::action::<serde_json::Value>(&format!("/me/invitations/{id}/accept")).await {
                                        Ok(v) => {
                                            state::load_me().await;
                                            toast(tr("Willkommen in der Organisation."));
                                            if let Some(org) = v["organizationId"].as_str() {
                                                nav.push(Route::Documents { org: org.to_string(), q: String::new() });
                                            }
                                        }
                                        Err(e) => toast_error(e),
                                    }
                                });
                            } },
                            {tr("Annehmen")}
                        }
                        button { class: "{btn(BTN_OUTLINE)} {BTN_SM}",
                            onclick: { let (id, name) = (inv.id.clone(), inv.organization_name.clone()); move |_| {
                                let (id, name) = (id.clone(), name.clone());
                                spawn(async move {
                                    if confirm(Ask::new(trf("Einladung zu „{0}“ ablehnen?", &[&(name)]), tr("Du kannst später erneut eingeladen werden.")).confirm(tr("Ablehnen")).destructive()).await {
                                        match api::delete(&format!("/me/invitations/{id}")).await {
                                            Ok(()) => { state::load_invitations().await; }
                                            Err(e) => toast_error(e),
                                        }
                                    }
                                });
                            } },
                            {tr("Ablehnen")}
                        }
                    }
                }
            }
        }
    }
}

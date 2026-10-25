//! Einstellungen (über das Profilmenü unten in der Seitenleiste): Konto, Organisation und –
//! für Administratoren – die Instanz samt KI/OpenRouter.

mod account;
mod admin;
mod org;

use crate::state::{ME, OVERVIEW};
use crate::Route;
use dioxus::prelude::*;
use crate::i18n::tr;

struct Item {
    key: &'static str,
    label: &'static str,
    group: &'static str,
}

const ITEMS: &[Item] = &[
    Item { key: "profile", label: "Profil & Passwort", group: "Konto" },
    Item { key: "api-keys", label: "API-Keys", group: "Konto" },
    Item { key: "scanner", label: "Scanner", group: "Konto" },
    Item { key: "mail", label: "E-Mail-Eingang", group: "Konto" },
    Item { key: "organization", label: "Allgemein & Mitglieder", group: "Organisation" },
    Item { key: "tagging-rules", label: "Tagging-Regeln", group: "Organisation" },
    Item { key: "properties", label: "Eigenschaften", group: "Organisation" },
    Item { key: "webhooks", label: "Webhooks", group: "Organisation" },
    Item { key: "ai", label: "KI & Modelle", group: "Instanz" },
    Item { key: "instance", label: "Registrierung & Benutzer", group: "Instanz" },
    Item { key: "printers", label: "Drucker", group: "Instanz" },
    Item { key: "watch-folders", label: "Ordner-Eingang", group: "Instanz" },
];

fn icon(key: &str) -> Element {
    match key {
        "profile" => rsx! { icons::User {} },
        "api-keys" => rsx! { icons::Key {} },
        "scanner" => rsx! { icons::ScanLine {} },
        "mail" => rsx! { icons::Mail {} },
        "organization" => rsx! { icons::Building2 {} },
        "tagging-rules" => rsx! { icons::Wand {} },
        "properties" => rsx! { icons::SlidersHorizontal {} },
        "webhooks" => rsx! { icons::Webhook {} },
        "ai" => rsx! { icons::Sparkles {} },
        "printers" => rsx! { icons::Printer {} },
        "watch-folders" => rsx! { icons::FolderInput {} },
        _ => rsx! { icons::ShieldCheck {} },
    }
}

#[component]
pub fn Settings(org: String, section: String) -> Element {
    let admin = ME.read().as_ref().is_some_and(|m| m.user.is_admin);
    let role = OVERVIEW.read().clone().filter(|(o, _)| *o == org).map(|(_, v)| v.role).unwrap_or_default();
    let manager = matches!(role.as_str(), "owner" | "admin");
    let visible: Vec<&Item> = ITEMS
        .iter()
        .filter(|i| i.group != {tr("Instanz")} || admin)
        .filter(|i| i.key != "webhooks" || manager)
        .collect();
    let mut groups: Vec<&str> = Vec::new();
    for i in &visible {
        if !groups.contains(&i.group) {
            groups.push(i.group);
        }
    }
    let title = visible.iter().find(|i| i.key == section).map(|i| tr(i.label)).unwrap_or_else(|| tr("Einstellungen"));
    rsx! {
        div { class: "mx-auto max-w-6xl p-4 md:p-6",
            div { class: "mb-6",
                h1 { class: "text-2xl font-semibold tracking-tight", {tr("Einstellungen")} }
                p { class: "mt-1 text-sm text-muted-foreground", "{title}" }
            }
            div { class: "grid gap-6 lg:grid-cols-[14rem_minmax(0,1fr)]",
                nav { class: "flex gap-1 overflow-x-auto lg:flex-col lg:overflow-visible",
                    for g in groups.iter().copied() {
                        p { class: "hidden px-2.5 pb-1 pt-3 text-xs font-medium uppercase tracking-wider text-muted-foreground first:pt-0 lg:block", {tr(g)} }
                        for i in visible.iter().filter(|i| i.group == g) {
                            Link { to: Route::Settings { org: org.clone(), section: i.key.to_string() },
                                class: if i.key == section { "flex shrink-0 items-center gap-2.5 rounded-md bg-accent px-2.5 py-1.5 text-sm font-medium [&_svg]:size-4" } else { "flex shrink-0 items-center gap-2.5 rounded-md px-2.5 py-1.5 text-sm text-muted-foreground hover:bg-accent/50 hover:text-foreground [&_svg]:size-4" },
                                {icon(i.key)}
                                {tr(i.label)}
                            }
                        }
                    }
                }
                div { class: "min-w-0 space-y-6",
                    match section.as_str() {
                        "profile" => rsx! { account::Profile {} },
                        "api-keys" => rsx! { account::ApiKeys {} },
                        "scanner" => rsx! { account::Scanner {} },
                        "mail" => rsx! { account::Mail { org: org.clone() } },
                        "organization" => rsx! { org::Organization { org: org.clone(), role: role.clone() } },
                        "tagging-rules" => rsx! { org::TaggingRules { org: org.clone() } },
                        "properties" => rsx! { org::Properties { org: org.clone() } },
                        "webhooks" if manager => rsx! { org::Webhooks { org: org.clone() } },
                        "ai" if admin => rsx! { admin::Ai {} },
                        "instance" if admin => rsx! { admin::Instance {} },
                        "printers" if admin => rsx! { admin::PrintersSettings {} },
                        "watch-folders" if admin => rsx! { admin::FoldersSettings { org: org.clone() } },
                        _ => rsx! { crate::components::common::Callout { text: {tr("Diesen Bereich gibt es nicht oder er ist nur für Administratoren.")} } },
                    }
                }
            }
        }
    }
}

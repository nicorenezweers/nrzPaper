//! Sprachwahl: als Auswahlfeld (Anmeldeseiten) und als Dialog (Profilmenü).

use crate::components::overlay::Modal;
use crate::i18n::{self, tr, LANG, LANGUAGES};
use dioxus::prelude::*;

/// Name der aktuellen Sprache in der Sprache selbst.
pub fn current_name() -> &'static str {
    let lang = LANG.read().clone();
    LANGUAGES.iter().find(|(c, _)| *c == lang).map(|(_, n)| *n).unwrap_or("Deutsch")
}

/// Kleines Auswahlfeld mit Globus.
#[component]
pub fn LanguageSelect() -> Element {
    rsx! {
        label { class: "inline-flex items-center gap-1.5 text-xs text-muted-foreground",
            icons::Languages { class: "size-3.5" }
            span { class: "sr-only", {tr("Sprache")} }
            select {
                class: "cursor-pointer rounded-md border-0 bg-transparent py-0.5 pr-1 text-xs text-muted-foreground outline-none hover:text-foreground [&>option]:bg-popover",
                onchange: move |e| {
                    let code = e.value();
                    spawn(async move { i18n::set_language(&code, true).await; });
                },
                for (code , name) in LANGUAGES.iter().copied() {
                    option { value: code, selected: *LANG.read() == code, "{name}" }
                }
            }
        }
    }
}

/// Dialog mit allen Sprachen.
#[component]
pub fn LanguageDialog(open: Signal<bool>) -> Element {
    rsx! {
        Modal { open, title: tr("Sprache"), description: tr("Gilt für diesen Browser."),
            div { class: "grid grid-cols-2 gap-1.5 sm:grid-cols-3",
                for (code , name) in LANGUAGES.iter().copied() {
                    button {
                        class: if *LANG.read() == code { "flex items-center justify-between rounded-md border border-primary bg-accent px-3 py-2 text-left text-sm" } else { "flex items-center justify-between rounded-md border px-3 py-2 text-left text-sm hover:bg-accent/60" },
                        onclick: move |_| {
                            spawn(async move {
                                i18n::set_language(code, true).await;
                                open.set(false);
                            });
                        },
                        span { "{name}" }
                        span { class: "font-mono text-[10px] uppercase text-muted-foreground", "{code}" }
                    }
                }
            }
        }
    }
}

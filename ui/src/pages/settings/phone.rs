//! Handy & Teilen: nrzPaper als Web-App installieren und PDFs aus anderen Apps teilen.
//! Android kann das direkt (Web Share Target im Manifest). Das iPhone lässt Web-Apps nicht
//! ins Teilen-Menü – dort übernimmt ein Kurzbefehl, der die Datei an einen persönlichen
//! Link schickt (API-Key nur mit „documents:create“, Ziel: die Inbox dieser Organisation).

use crate::api;
use crate::components::common::*;
use crate::components::overlay::*;
use crate::i18n::tr;
use crate::util;
use dioxus::prelude::*;
use serde_json::json;

#[component]
fn Step(n: usize, #[props(into)] title: String, children: Element) -> Element {
    rsx! {
        li { class: "flex gap-3",
            span { class: "grid size-7 shrink-0 place-items-center rounded-full bg-primary text-sm font-semibold text-primary-foreground", "{n}" }
            div { class: "min-w-0 flex-1 pt-0.5",
                p { class: "font-medium", "{title}" }
                div { class: "mt-1 text-sm text-muted-foreground", {children} }
            }
        }
    }
}

#[component]
pub fn Phone(org: String) -> Element {
    let mut link = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let org_name = crate::state::ME.read().as_ref().and_then(|m| m.organizations.iter().find(|o| o.id == org).map(|o| o.name.clone())).unwrap_or_default();

    let org_key = org.clone();
    let create = move |_| {
        let org = org_key.clone();
        spawn(async move {
            busy.set(true);
            let body = json!({ "name": tr("iPhone – Teilen"), "permissions": ["documents:create"] });
            let res: api::ApiResult<serde_json::Value> = api::post("/me/api-keys", &body).await;
            busy.set(false);
            match res {
                Ok(v) => {
                    let key = v["key"].as_str().unwrap_or_default();
                    link.set(util::absolute(&format!("/api/v1/organizations/{org}/review/drop?key={key}")));
                }
                Err(e) => toast_error(e),
            }
        });
    };
    let copy = move |_| {
        spawn(async move {
            if util::copy(&link()).await {
                toast(tr("Link kopiert."));
            } else {
                toast_error(tr("Kopieren ging nicht – Link lange antippen und kopieren."));
            }
        });
    };

    rsx! {
        Section { title: tr("Als App auf dem Handy"), description: tr("nrzPaper läuft als Web-App – ohne App Store, mit eigenem Symbol auf dem Home-Bildschirm."),
            ol { class: "grid gap-4",
                Step { n: 1, title: tr("iPhone: in Safari öffnen"),
                    {tr("Unten auf Teilen tippen (Quadrat mit Pfeil) → „Zum Home-Bildschirm“ → „Hinzufügen“.")}
                }
                Step { n: 2, title: tr("Android: in Chrome öffnen"),
                    {tr("Menü (⋮) → „App installieren“. Danach steht nrzPaper direkt im Teilen-Menü – PDFs und Fotos einfach an nrzPaper teilen.")}
                }
                Step { n: 3, title: tr("Scannen"),
                    {tr("Unten in der Mitte auf die Kamera tippen, Seite fotografieren, fertig. Mehrere Seiten werden ein PDF.")}
                }
            }
        }

        Section { title: tr("PDFs vom iPhone teilen"),
            description: tr("Apple lässt Web-Apps nicht ins Teilen-Menü. Ein Kurzbefehl übernimmt das – einmal einrichten, dauert etwa eine Minute."),
            ol { class: "grid gap-5",
                Step { n: 1, title: tr("Persönlichen Link erstellen"),
                    p { {tr("Dateien an diesen Link landen in der Inbox von")} " " span { class: "font-medium text-foreground", "{org_name}" } "." }
                    if link().is_empty() {
                        button { class: "{btn(BTN_PRIMARY)} mt-3", disabled: busy(), onclick: create, icons::Link { class: "size-4" } {tr("Link erstellen")} }
                    } else {
                        div { class: "mt-3 grid gap-2 sm:flex",
                            input { class: "{INPUT} font-mono text-xs", readonly: true, value: "{link}", onfocus: move |_| { document::eval("document.activeElement?.select?.()"); } }
                            button { class: btn(BTN_PRIMARY), onclick: copy, icons::Copy { class: "size-4" } {tr("Kopieren")} }
                        }
                        p { class: "mt-2 text-xs", {tr("Der Link ist wie ein Passwort – nicht weitergeben. Widerrufen unter Einstellungen → API-Keys („iPhone – Teilen“).")} }
                    }
                }
                Step { n: 2, title: tr("Kurzbefehl anlegen"),
                    {tr("App „Kurzbefehle“ öffnen → „+“. Oben auf den Namen tippen → „Umbenennen“: nrzPaper. Dann „ⓘ“ (Details) → „Im Share-Sheet anzeigen“ einschalten.")}
                }
                Step { n: 3, title: tr("Aktion „Inhalte von URL abrufen“"),
                    {tr("Aktion suchen und hinzufügen, als URL den kopierten Link einfügen.")}
                }
                Step { n: 4, title: tr("Datei mitschicken"),
                    {tr("In der Aktion auf den Pfeil (Mehr anzeigen) tippen: Methode „POST“, Anfragetext „Datei“, bei Datei „Kurzbefehleingabe“ wählen. Fertig.")}
                }
            }
            div { class: "mt-5 flex items-start gap-3 rounded-lg border bg-background/50 p-3 text-sm",
                icons::Share { class: "mt-0.5 size-4 shrink-0" }
                span { {tr("Ab jetzt in jeder App (Mail, Dateien, Safari …): Teilen → nrzPaper. Das Dokument landet in der Inbox, die KI schlägt Name und Tags vor.")} }
            }
        }
    }
}

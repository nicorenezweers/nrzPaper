//! Dialog, Bestätigung, Menü und Meldungen – Aussehen von rust-ui (Dialog, Alert Dialog,
//! Dropdown Menu, Toast), der Zustand liegt hier in Dioxus-Signalen statt in eingeschleustem
//! JavaScript. Ersetzt überall die Browser-Popups (`confirm`, `alert`, `prompt`).

use dioxus::prelude::*;
use futures_channel::oneshot;
use std::cell::RefCell;
use std::rc::Rc;
use tw_merge::tw_merge;
use crate::i18n::tr;

// ------------------------------------------------------------------ Dialog

/// Modales Fenster. Schliesst mit Esc, Klick daneben (abschaltbar) oder dem X.
#[component]
pub fn Modal(
    open: Signal<bool>,
    #[props(into)] title: String,
    #[props(into, optional)] description: Option<String>,
    #[props(into, optional)] class: Option<String>,
    #[props(default = true)] dismissible: bool,
    children: Element,
) -> Element {
    if !open() {
        return rsx! {};
    }
    let content = tw_merge!(
        "relative bg-background border rounded-2xl shadow-lg p-6 w-full max-w-lg max-h-[85vh] overflow-y-auto animate-in",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        div {
            class: "fixed inset-0 z-[60] flex items-center justify-center bg-black/60 p-4 backdrop-blur-[2px]",
            tabindex: "-1",
            // Erstes Eingabefeld des Dialogs fokussieren, sonst die Fläche selbst (für Escape)
            onmounted: move |_| {
                document::eval(
                    r#"requestAnimationFrame(() => {
                        const d = [...document.querySelectorAll('[role=dialog]')].pop();
                        if (!d) return;
                        const f = d.querySelector('[autofocus], input:not([type=hidden]):not([readonly]):not([disabled]), textarea:not([readonly]), select:not([disabled])');
                        (f || d.parentElement).focus();
                    })"#,
                );
            },
            onkeydown: move |e| if e.key() == Key::Escape && dismissible { open.set(false) },
            onclick: move |_| if dismissible { open.set(false) },
            div {
                class: "{content}",
                role: "dialog",
                "aria-modal": "true",
                onclick: move |e| e.stop_propagation(),
                button {
                    class: "absolute top-4 right-4 rounded-sm p-1 text-muted-foreground hover:text-foreground focus:outline-none focus:ring-2 focus:ring-ring",
                    title: tr("Schliessen"),
                    onclick: move |_| open.set(false),
                    icons::X { class: "size-4" }
                }
                div { class: "flex flex-col gap-2 pr-6",
                    h3 { class: "text-lg leading-none font-semibold", "{title}" }
                    if let Some(d) = description {
                        p { class: "text-muted-foreground text-sm", "{d}" }
                    }
                }
                div { class: "mt-5", {children} }
            }
        }
    }
}

// ------------------------------------------------------------------ Bestätigung

#[derive(Clone)]
pub struct Ask {
    pub title: String,
    pub description: String,
    pub confirm: String,
    pub destructive: bool,
}

impl Ask {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Ask { title: title.into(), description: description.into(), confirm: {tr("Bestätigen")}.into(), destructive: false }
    }
    pub fn confirm(mut self, label: impl Into<String>) -> Self {
        self.confirm = label.into();
        self
    }
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}

type Pending = Rc<RefCell<Option<oneshot::Sender<bool>>>>;

static ASK: GlobalSignal<Option<(Ask, Pending)>> = Signal::global(|| None);

/// Eigene Rückfrage statt `window.confirm` – `true`, wenn bestätigt.
pub async fn confirm(ask: Ask) -> bool {
    let (tx, rx) = oneshot::channel();
    if let Some((_, old)) = ASK.write().replace((ask, Rc::new(RefCell::new(Some(tx))))) {
        if let Some(s) = old.borrow_mut().take() {
            let _ = s.send(false);
        }
    }
    rx.await.unwrap_or(false)
}

fn answer(yes: bool) {
    if let Some((_, pending)) = ASK.write().take() {
        if let Some(s) = pending.borrow_mut().take() {
            let _ = s.send(yes);
        }
    }
}

/// Einmal in der App einbinden.
#[component]
pub fn ConfirmHost() -> Element {
    let Some((ask, _)) = ASK.read().clone() else { return rsx! {} };
    let action = if ask.destructive {
        "bg-destructive text-white hover:bg-destructive/90"
    } else {
        "bg-primary text-primary-foreground hover:bg-primary/90"
    };
    rsx! {
        div {
            class: "fixed inset-0 z-[70] flex items-center justify-center bg-black/60 p-4 backdrop-blur-[2px]",
            tabindex: "-1",
            onmounted: move |e| async move { let _ = e.set_focus(true).await; },
            onkeydown: move |e| match e.key() {
                Key::Escape => answer(false),
                Key::Enter => answer(true),
                _ => {}
            },
            div { class: "w-full max-w-md rounded-2xl border bg-background p-6 shadow-lg animate-in", role: "alertdialog",
                div { class: "flex gap-4",
                    if ask.destructive {
                        div { class: "grid size-10 shrink-0 place-items-center rounded-full bg-destructive/15 text-destructive",
                            icons::TriangleAlert { class: "size-5" }
                        }
                    }
                    div { class: "flex flex-col gap-2",
                        h3 { class: "text-lg leading-none font-semibold", "{ask.title}" }
                        p { class: "text-muted-foreground text-sm", "{ask.description}" }
                    }
                }
                footer { class: "mt-6 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
                    button {
                        class: "inline-flex h-9 items-center justify-center rounded-md border bg-background px-4 text-sm font-medium shadow-xs hover:bg-accent dark:bg-input/30 dark:border-input",
                        onclick: move |_| answer(false),
                        {tr("Abbrechen")}
                    }
                    button {
                        class: "inline-flex h-9 items-center justify-center rounded-md px-4 text-sm font-medium shadow-xs {action}",
                        onclick: move |_| answer(true),
                        "{ask.confirm}"
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ Meldungen

#[derive(Clone, PartialEq)]
struct Toast {
    id: u64,
    text: String,
    error: bool,
    /// Wird gerade ausgeblendet (Animation läuft, danach entfernt)
    leaving: bool,
}

static TOASTS: GlobalSignal<Vec<Toast>> = Signal::global(Vec::new);
static NEXT: GlobalSignal<u64> = Signal::global(|| 0);

/// Dauer der Ausblende-Animation (muss zu `toast-out` in tailwind.css passen).
const LEAVE_MS: u32 = 220;

fn push(text: String, error: bool) {
    let id = {
        let mut n = NEXT.write();
        *n += 1;
        *n
    };
    TOASTS.write().push(Toast { id, text, error, leaving: false });
    spawn(async move {
        gloo_timers::future::TimeoutFuture::new(if error { 7000 } else { 3500 }).await;
        dismiss(id);
    });
}

/// Meldung sanft ausblenden und danach entfernen.
fn dismiss(id: u64) {
    let mut found = false;
    for t in TOASTS.write().iter_mut().filter(|t| t.id == id && !t.leaving) {
        t.leaving = true;
        found = true;
    }
    if found {
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(LEAVE_MS).await;
            TOASTS.write().retain(|t| t.id != id);
        });
    }
}

/// Kurze Meldung unten rechts.
pub fn toast(text: impl Into<String>) {
    push(text.into(), false);
}

/// Fehlermeldung (bleibt länger stehen).
pub fn toast_error(text: impl std::fmt::Display) {
    push(text.to_string(), true);
}

#[component]
pub fn Toaster() -> Element {
    rsx! {
        div { class: "pointer-events-none fixed bottom-20 right-4 z-[80] flex w-full max-w-sm flex-col gap-2",
            for t in TOASTS.read().iter().cloned() {
                div {
                    key: "{t.id}",
                    role: if t.error { "alert" } else { "status" },
                    class: format!(
                        "pointer-events-auto flex items-start gap-3 rounded-xl border bg-popover px-4 py-3 text-sm text-popover-foreground shadow-lg {} {}",
                        if t.error { "border-destructive/40" } else { "" },
                        if t.leaving { "animate-toast-out" } else { "animate-toast-in" },
                    ),
                    if t.error {
                        icons::CircleAlert { class: "mt-0.5 size-4 shrink-0 text-destructive" }
                    } else {
                        icons::Check { class: "mt-0.5 size-4 shrink-0 text-muted-foreground" }
                    }
                    span { class: "flex-1", "{t.text}" }
                    button {
                        class: "text-muted-foreground hover:text-foreground",
                        onclick: move |_| dismiss(t.id),
                        icons::X { class: "size-3.5" }
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ Menü

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Align {
    #[default]
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Side {
    #[default]
    Bottom,
    Top,
}

/// Aufklappmenü (rust-ui Dropdown Menu). `trigger` ist der sichtbare Knopf.
#[component]
pub fn Dropdown(
    trigger: Element,
    #[props(default)] align: Align,
    #[props(default)] side: Side,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] trigger_class: Option<String>,
    children: Element,
) -> Element {
    let mut open = use_signal(|| false);
    provide_context(MenuOpen(open));
    let pos = match (align, side) {
        (Align::Start, Side::Bottom) => "left-0 top-full mt-1.5",
        (Align::End, Side::Bottom) => "right-0 top-full mt-1.5",
        (Align::Start, Side::Top) => "left-0 bottom-full mb-1.5",
        (Align::End, Side::Top) => "right-0 bottom-full mb-1.5",
    };
    let menu = tw_merge!(
        "absolute z-50 min-w-[12rem] overflow-hidden rounded-lg border bg-popover p-1 text-popover-foreground shadow-lg animate-in",
        pos,
        class.as_deref().unwrap_or("")
    );
    rsx! {
        div { class: "relative",
            button {
                class: trigger_class.unwrap_or_default(),
                "aria-expanded": "{open}",
                onclick: move |_| open.toggle(),
                {trigger}
            }
            if open() {
                div { class: "fixed inset-0 z-40", onclick: move |_| open.set(false) }
                div { class: "{menu}", role: "menu", {children} }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct MenuOpen(Signal<bool>);

/// Eintrag im Menü; schliesst es nach dem Klick.
#[component]
pub fn MenuItem(
    onclick: EventHandler<MouseEvent>,
    #[props(default)] destructive: bool,
    #[props(default)] active: bool,
    children: Element,
) -> Element {
    let ctx = try_consume_context::<MenuOpen>();
    let color = if destructive { "text-destructive hover:bg-destructive/10" } else { "hover:bg-accent hover:text-accent-foreground" };
    let act = if active { "bg-accent/60" } else { "" };
    rsx! {
        button {
            class: "relative flex w-full cursor-default select-none items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm outline-none [&_svg]:size-4 [&_svg]:shrink-0 {color} {act}",
            role: "menuitem",
            onclick: move |e| {
                if let Some(MenuOpen(mut o)) = ctx {
                    o.set(false);
                }
                onclick.call(e);
            },
            {children}
        }
    }
}

#[component]
pub fn MenuLabel(children: Element) -> Element {
    rsx! { div { class: "px-2 py-1.5 text-xs font-medium text-muted-foreground", {children} } }
}

#[component]
pub fn MenuSeparator() -> Element {
    rsx! { div { class: "-mx-1 my-1 h-px bg-border" } }
}

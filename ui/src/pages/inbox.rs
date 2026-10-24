use dioxus::prelude::*;

#[component]
pub fn Inbox(org: String, item: String) -> Element {
    rsx! { div { class: "p-6", "Inbox" } }
}

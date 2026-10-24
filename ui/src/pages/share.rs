use dioxus::prelude::*;

#[component]
pub fn Share(token: String) -> Element {
    rsx! { div { class: "p-6", "Share" } }
}

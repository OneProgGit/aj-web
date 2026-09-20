use dioxus::prelude::*;

use super::icon::{icon_element, Icon};

/// Back link with the «назад» label. Mirrors the `BackButton` used on aj-app pages.
#[component]
pub fn BackButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "btn btn-ghost btn-sm gap-2",
            onclick: move |ev| onclick.call(ev),
            {icon_element(Icon::Back, 16)}
            span { "{crate::i18n::tr(&crate::state::language(), \"назад\", \"back\")}" }
        }
    }
}

use dioxus::prelude::*;

use super::icon::{Icon, icon_slot};

/// Back link with the «назад» label. Mirrors the `BackButton` used on aj-app pages.
#[component]
pub fn BackButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        m3e-button {
            variant: "text",
            onclick: move |ev| onclick.call(ev),
            {icon_slot(Icon::Back, 16)}
            span { "{crate::i18n::tr(&crate::state::language(), \"назад\", \"back\")}" }
        }
    }
}

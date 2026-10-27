use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    i18n,
};

use super::icon::{Icon, icon_element};

/// Icon-only copy button. Kept short so it fits next to URLs and inputs.
#[component]
pub fn CopyButton(text: String) -> Element {
    let lang = crate::state::language();
    let label = i18n::tr(&lang, "копировать", "copy");
    rsx! {
        button {
            class: "btn btn-ghost btn-sm btn-square",
            r#type: "button",
            title: "{label}",
            "aria-label": "{label}",
            disabled: text.is_empty(),
            onclick: move |_| {
                crate::state::copy_text(&text);
                show_alert(
                    AlertKind::Success,
                    i18n::tr(&crate::state::language(), "Скопировано", "Copied"),
                );
            },
            {icon_element(Icon::Copy, 16)}
        }
    }
}

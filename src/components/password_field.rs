use dioxus::prelude::*;

use super::icon::{Icon, icon_slot};

/// Password input with a show/hide eye toggle, in the M3E outlined form field.
#[component]
pub fn PasswordField(
    value: Signal<String>,
    label: String,
    placeholder: String,
    class: String,
) -> Element {
    let mut show = use_signal(|| false);
    rsx! {
        m3e-form-field {
            span { slot: "label", "{label}" }
            m3e-icon-button {
                slot: "suffix",
                variant: "text",
                onclick: move |_| show.set(!show()),
                {icon_slot(if show() { Icon::EyeOff } else { Icon::Eye }, 16)}
            }
            input {
                class: "{class}",
                r#type: if show() { "text" } else { "password" },
                placeholder: "{placeholder}",
                value: value(),
                oninput: move |ev| value.set(ev.value()),
            }
        }
    }
}

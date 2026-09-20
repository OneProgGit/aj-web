use dioxus::prelude::*;

use super::icon::{icon_element, Icon};

/// Password input with a show/hide eye toggle.
#[component]
pub fn PasswordField(value: Signal<String>, placeholder: String, class: String) -> Element {
    let mut show = use_signal(|| false);
    rsx! {
        div { class: "relative w-full max-w-md",
            input {
                class: "{class} w-full pr-10",
                r#type: if show() { "text" } else { "password" },
                placeholder: "{placeholder}",
                value: value(),
                oninput: move |ev| value.set(ev.value()),
            }
            button {
                class: "btn btn-ghost btn-sm btn-circle absolute right-1 top-1/2 -translate-y-1/2",
                onclick: move |_| show.set(!show()),
                {icon_element(if show() { Icon::EyeOff } else { Icon::Eye }, 16)}
            }
        }
    }
}

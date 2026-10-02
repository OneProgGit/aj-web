use dioxus::prelude::*;

use crate::{components::icon::Icon, components::icon::icon_slot, i18n};

/// Landing page («Добро пожаловать!») with login / register entry points.
/// The app-level router redirects authenticated users straight to /home.
#[component]
pub fn Welcome() -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();

    rsx! {
        div { class: "flex flex-col items-start gap-4 max-w-7xl mx-auto w-full",
            h1 { class: "text-3xl font-bold", "{i18n::tr(&lang, \"Добро пожаловать!\", \"Welcome!\")}" }

            m3e-button {
                variant: "filled",
                onclick: move |_| { let _ = navigator.push(crate::Route::Login {}); },
                {icon_slot(Icon::Enter, 16)}
                span { "{i18n::tr(&lang, \"войти\", \"log in\")}" }
            }

            m3e-button {
                variant: "text",
                onclick: move |_| { let _ = navigator.push(crate::Route::Register {}); },
                {icon_slot(Icon::Plus, 16)}
                span { "{i18n::tr(&lang, \"создать аккаунт\", \"create account\")}" }
            }
        }
    }
}

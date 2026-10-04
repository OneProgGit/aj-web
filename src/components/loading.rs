use dioxus::prelude::*;

use crate::i18n;

/// Спиннер с подписью «загрузка» — единый вид для всех страниц.
#[component]
pub fn Loading() -> Element {
    let lang = crate::state::language();
    rsx! {
        div { class: "flex flex-col justify-center items-center gap-3 py-10",
            span { class: "loading loading-spinner loading-lg" }
            p { class: "italic", "{i18n::tr(&lang, \"загрузка\", \"loading\")}" }
        }
    }
}

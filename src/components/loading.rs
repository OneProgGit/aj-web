use dioxus::prelude::*;

use crate::i18n;

/// Спиннер с подписью «загрузка» — единый вид для всех страниц.
/// `w-full` обязателен: на страницах с `items-start` контейнер иначе
/// сжимает блок по контенту и прижимает спиннер к левому краю.
#[component]
pub fn Loading() -> Element {
    let lang = crate::state::language();
    rsx! {
        div { class: "w-full flex flex-col justify-center items-center gap-3 py-10",
            m3e-loading-indicator { }
            p { class: "italic", "{i18n::tr(&lang, \"загрузка\", \"loading\")}" }
        }
    }
}

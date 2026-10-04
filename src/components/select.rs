use dioxus::prelude::*;

/// Выпадающий список на daisyUI. Значение приходит обычным
/// `Event::value()` — Dioxus кастует цель к `HtmlSelectElement`.
#[component]
pub fn M3Select(
    id: String,
    label: String,
    options: Vec<(String, String)>,
    value: String,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        label { class: "flex flex-col w-full max-w-xs",
            div { class: "pb-1",
                span { class: "text-sm opacity-75", "{label}" }
            }
            select {
                id: "{id}",
                class: "select select-bordered w-full",
                onchange: move |ev: Event<FormData>| onchange.call(ev.value()),
                for option in options {
                    option { value: "{option.0}", selected: option.0 == value, "{option.1}" }
                }
            }
        }
    }
}

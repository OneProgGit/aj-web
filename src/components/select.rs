use dioxus::prelude::*;

/// Читает значение `m3e-select` по его `id`.
///
/// У кастомных элементов Dioxus не умеет брать значение из события:
/// `Event::value()` кастует цель к `HtmlSelectElement`/`HtmlInputElement`
/// и для кастомного тега всегда отдаёт пустое.
fn read_select_value(id: &str, onchange: EventHandler<String>) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(el) = document.get_element_by_id(id) else {
        return;
    };
    if let Ok(v) = js_sys::Reflect::get(&el, &wasm_bindgen::JsValue::from_str("value"))
        && let Some(s) = v.as_string()
    {
        onchange.call(s);
    }
}

/// Выпадающий список на `m3e-select` внутри `m3e-form-field`.
#[component]
pub fn M3Select(
    id: String,
    label: String,
    options: Vec<(String, String)>,
    value: String,
    onchange: EventHandler<String>,
) -> Element {
    let id_a = id.clone();
    let id_b = id.clone();

    rsx! {
        m3e-form-field {
            span { slot: "label", "{label}" }
            m3e-select {
                id: "{id}",
                value: "{value}",
                oninput: move |_ev: Event<FormData>| read_select_value(&id_a, onchange),
                onchange: move |_ev: Event<FormData>| read_select_value(&id_b, onchange),
                for option in options {
                    m3e-option {
                        value: "{option.0}",
                        selected: option.0 == value,
                        "{option.1}"
                    }
                }
            }
        }
    }
}

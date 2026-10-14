use dioxus::prelude::*;

use crate::{
    components::icon::{Icon, icon_element},
    i18n,
};

/// 1-based номер из интерфейса -> 0-based индекс для запроса.
#[must_use]
pub fn problem_index_zero_based(number: i64) -> i64 {
    number.saturating_sub(1).max(0)
}

/// Степпер номера задачи в контесте.
///
/// В интерфейсе номер 1-based (как его показывает таблица результатов), а на
/// проводе сервер ждёт 0-based, поэтому наружу отдаём 1-based значение, а перед
/// отправкой переводим через [`problem_index_zero_based`].
#[component]
pub fn ProblemNumber(number: Signal<i64>, min: i64, disabled: bool) -> Element {
    let lang = crate::state::language();

    rsx! {
        div { class: "flex flex-col gap-1",
            span { class: "text-sm opacity-70", "{i18n::tr(&lang, \"Номер задачи в контесте\", \"Problem number in contest\")}" }
            div { class: "join",
                button {
                    class: "btn btn-sm join-item",
                    r#type: "button",
                    disabled,
                    onclick: move |_| {
                        number.set(number().saturating_sub(1).max(min));
                    },
                    {icon_element(Icon::Minus, 16)}
                }
                input {
                    class: "input input-sm input-bordered join-item w-20 text-center font-mono",
                    r#type: "number",
                    min: "{min}",
                    step: "1",
                    disabled,
                    value: "{number()}",
                    onchange: move |ev| {
                        // Пустое или мусорное поле не должно обнулять номер:
                        // оставляем прежний, чтобы не отправить серверу мусор.
                        if let Ok(parsed) = ev.value().parse::<i64>() {
                            number.set(parsed.max(min));
                        }
                    },
                }
                button {
                    class: "btn btn-sm join-item",
                    r#type: "button",
                    disabled,
                    onclick: move |_| {
                        number.set(number().saturating_add(1).max(min));
                    },
                    {icon_element(Icon::Plus, 16)}
                }
            }
        }
    }
}

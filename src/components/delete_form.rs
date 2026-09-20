use dioxus::prelude::*;

use super::{
    icon::{icon_element, Icon},
    password_field::PasswordField,
};

/// Mirrors the «Требуется рут-доступ» popup used on contest/post/question/
/// problem/account deletion in aj-app. Calls `on_delete(login, password,
/// confirmation)` when the «удалить» button is clicked and all fields are valid.
#[component]
pub fn DeleteForm(
    on_delete: EventHandler<(String, String, bool)>,
    on_cancel: EventHandler<MouseEvent>,
) -> Element {
    let mut login = use_signal(String::new);
    let password = use_signal(String::new);
    let mut confirmed = use_signal(|| false);

    rsx! {
        div { class: "modal modal-open",
            div { class: "modal-box max-w-sm text-base-content",
                div { class: "flex flex-col gap-3",
                    div { class: "flex items-center justify-between gap-4",
                        p { class: "card-title", "{crate::i18n::tr(&crate::state::language(), \"Требуется рут-доступ\", \"Root access required\")}" }
                        button {
                            class: "btn btn-sm btn-circle btn-ghost",
                            onclick: move |ev| on_cancel.call(ev),
                            "✕"
                        }
                    }
                    input {
                        class: "input input-bordered",
                        placeholder: crate::i18n::tr(&crate::state::language(), "логин", "login"),
                        value: login(),
                        oninput: move |ev| login.set(ev.value()),
                    }
                    PasswordField {
                        value: password,
                        placeholder: crate::i18n::tr(&crate::state::language(), "пароль", "password"),
                        class: "input input-bordered".to_string(),
                    }
                    label { class: "label cursor-pointer justify-start gap-2",
                        input {
                            r#type: "checkbox",
                            class: "checkbox checkbox-error checkbox-sm",
                            checked: confirmed(),
                            onchange: move |ev| confirmed.set(ev.checked()),
                        }
                        span { class: "label-text", "{crate::i18n::tr(&crate::state::language(), \"подтвердите удаление\", \"confirm deletion\")}" }
                    }
                    div { class: "card-actions justify-end mt-2",
                        button {
                            class: "btn btn-ghost btn-sm",
                            onclick: move |ev| on_cancel.call(ev),
                            span { "{crate::i18n::tr(&crate::state::language(), \"отменить\", \"cancel\")}" }
                        }
                        button {
                            class: "btn btn-error btn-sm gap-2",
                            disabled: !(!login().is_empty() && !password().is_empty() && confirmed()),
                            onclick: {
                                let on_delete = on_delete;
                                move |_| on_delete.call((login(), password(), confirmed()))
                            },
                            {icon_element(if confirmed() { Icon::Trash } else { Icon::CircleBackslash }, 16)}
                            span { "{crate::i18n::tr(&crate::state::language(), \"удалить\", \"delete\")}" }
                        }
                    }
                }
            }
        }
    }
}

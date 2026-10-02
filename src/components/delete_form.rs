use dioxus::prelude::*;

use super::{
    icon::{Icon, icon_slot},
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
                        m3e-icon-button {
                            onclick: move |ev| on_cancel.call(ev),
                            "✕"
                        }
                    }
                    m3e-form-field {
                        input {
                            placeholder: crate::i18n::tr(&crate::state::language(), "логин", "login"),
                            value: login(),
                            oninput: move |ev| login.set(ev.value()),
                        }
                    }
                    PasswordField {
                        value: password,
                        label: crate::i18n::tr(&crate::state::language(), "пароль", "password"),
                        placeholder: crate::i18n::tr(&crate::state::language(), "не меньше 8 символов", "at least 8 characters"),
                        class: "w-full".to_string(),
                    }
                    label { class: "label cursor-pointer justify-start gap-2",
                        m3e-checkbox {
                            class: "text-error",
                            checked: confirmed(),
                            onchange: move |_| confirmed.set(!confirmed()),
                        }
                        span { class: "label-text", "{crate::i18n::tr(&crate::state::language(), \"подтвердите удаление\", \"confirm deletion\")}" }
                    }
                    div { class: "card-actions justify-end mt-2",
                        m3e-button {
                            variant: "text",
                            onclick: move |ev| on_cancel.call(ev),
                            span { "{crate::i18n::tr(&crate::state::language(), \"отменить\", \"cancel\")}" }
                        }
                        m3e-button {
                            variant: "filled",
                            class: "destructive",
                            disabled: !(!login().is_empty() && !password().is_empty() && confirmed()),
                            onclick: {
                                let on_delete = on_delete;
                                move |_| on_delete.call((login(), password(), confirmed()))
                            },
                            {icon_slot(if confirmed() { Icon::Trash } else { Icon::CircleBackslash }, 16)}
                            span { "{crate::i18n::tr(&crate::state::language(), \"удалить\", \"delete\")}" }
                        }
                    }
                }
            }
        }
    }
}

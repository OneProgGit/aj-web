use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    api,
    components::icon::icon_slot,
    components::{icon::Icon, password_field::PasswordField},
    i18n,
};

#[component]
pub fn Register() -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let mut login = use_signal(String::new);
    let password = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut busy = use_signal(|| false);

    let valid =
        use_memo(move || !login().is_empty() && !password().is_empty() && password() == confirm());

    rsx! {
        div { class: "flex flex-col items-start gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex gap-4 items-center",
                m3e-button {
                    variant: "text",
                    onclick: move |_| { let _ = navigator.push(crate::Route::Welcome {}); },
                    {icon_slot(Icon::Back, 16)}
                    span { "{i18n::tr(&lang, \"назад\", \"back\")}" }
                }
                h1 { class: "text-3xl font-bold", "{i18n::tr(&lang, \"Создание аккаунта\", \"Create account\")}" }
            }

            m3e-form-field {
                input {
                    placeholder: i18n::tr(&lang, "логин", "login"),
                    value: login(),
                    oninput: move |ev| login.set(ev.value()),
                }
            }

            PasswordField {
                value: password,
                label: i18n::tr(&lang, "пароль", "password"),
                placeholder: i18n::tr(&lang, "не меньше 8 символов", "at least 8 characters"),
                class: "w-full max-w-md".to_string(),
            }

            PasswordField {
                value: confirm,
                label: i18n::tr(&lang, "подтвердите пароль", "confirm password"),
                placeholder: i18n::tr(&lang, "ещё раз", "repeat"),
                class: "w-full max-w-md".to_string(),
            }

            m3e-button {
                variant: "filled",
                disabled: busy() || !valid(),
                onclick: {
                    let nav = navigator;
                    move |_| {
                        if !valid() || busy() {
                            return;
                        }
                        busy.set(true);
                        let login = login();
                        let password = password();
                        let confirm = confirm();
                        spawn(async move {
                            match api::auth::register(&login, &password, &confirm).await {
                                Ok(()) => {
                                    busy.set(false);
                                    show_alert(
                                        AlertKind::Success,
                                        i18n::tr("ru", "Аккаунт создан", "Account created"),
                                    );
                                    nav.push(crate::Route::Login {});
                                }
                                Err(e) => {
                                    busy.set(false);
                                    show_alert(AlertKind::Error, e);
                                }
                            }
                        });
                    }
                },
                {icon_slot(Icon::Plus, 16)}
                span { "{i18n::tr(&lang, \"создать аккаунт\", \"create account\")}" }
            }
        }
    }
}

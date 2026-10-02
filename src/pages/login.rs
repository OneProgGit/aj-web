use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    api,
    components::icon::icon_slot,
    components::{icon::Icon, password_field::PasswordField},
    i18n,
    state::STATE,
};

#[component]
pub fn Login() -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let mut login = use_signal(String::new);
    let password = use_signal(String::new);
    let mut busy = use_signal(|| false);

    rsx! {
        div { class: "flex flex-col items-start gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex gap-4 items-center",
                m3e-button {
                    variant: "text",
                    onclick: move |_| { let _ = navigator.push(crate::Route::Welcome {}); },
                    {icon_slot(Icon::Back, 16)}
                    span { "{i18n::tr(&lang, \"назад\", \"back\")}" }
                }
                h1 { class: "text-3xl font-bold", "{i18n::tr(&lang, \"Вход в аккаунт\", \"Log in\")}" }
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
                placeholder: i18n::tr(&lang, "ваш пароль", "your password"),
                class: "w-full max-w-md".to_string(),
            }

            m3e-button {
                variant: "filled",
                disabled: busy(),
                onclick: {
                    let nav = navigator;
                    move |_| {
                        if busy() {
                            return;
                        }
                        busy.set(true);
                        let login = login();
                        let password = password();
                        let nav = nav;
                        spawn(async move {
                            match api::auth::login(&login, &password).await {
                                Ok(token) => {
                                    crate::state::save_token(&token);
                                    STATE.write().token = Some(token);
                                    match api::users::get_me(&crate::state::token()).await {
                                        Ok(me) => {
                                            STATE.write().user = Some(me);
                                            busy.set(false);
                                            nav.push(crate::Route::Home {});
                                        }
                                        Err(e) => {
                                            busy.set(false);
                                            show_alert(AlertKind::Error, e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    busy.set(false);
                                    show_alert(AlertKind::Error, e);
                                }
                            }
                        });
                    }
                },
                {icon_slot(Icon::Enter, 16)}
                span { "{i18n::tr(&lang, \"войти\", \"log in\")}" }
            }
        }
    }
}

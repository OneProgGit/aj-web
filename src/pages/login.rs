use dioxus::prelude::*;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        icon::{icon_element, Icon},
        password_field::PasswordField,
    },
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
                button {
                    class: "btn btn-ghost btn-sm gap-2",
                    onclick: move |_| { let _ = navigator.push(crate::Route::Welcome {}); },
                    {icon_element(Icon::Back, 16)}
                    span { "{i18n::tr(&lang, \"назад\", \"back\")}" }
                }
                h1 { class: "text-3xl font-bold", "{i18n::tr(&lang, \"Вход в аккаунт\", \"Log in\")}" }
            }

            input {
                class: "input input-bordered w-full max-w-md",
                placeholder: i18n::tr(&lang, "логин", "login"),
                value: login(),
                oninput: move |ev| login.set(ev.value()),
            }

            PasswordField {
                value: password,
                placeholder: i18n::tr(&lang, "пароль", "password"),
                class: "input input-bordered max-w-md".to_string(),
            }

            button {
                class: "btn btn-primary btn-sm gap-1",
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
                {icon_element(Icon::Enter, 16)}
                span { "{i18n::tr(&lang, \"войти\", \"log in\")}" }
            }
        }
    }
}

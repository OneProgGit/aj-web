use dioxus::prelude::*;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        icon::{icon_element, Icon},
        password_field::PasswordField,
    },
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
                button {
                    class: "btn btn-ghost btn-sm gap-2",
                    onclick: move |_| { let _ = navigator.push(crate::Route::Welcome {}); },
                    {icon_element(Icon::Back, 16)}
                    span { "{i18n::tr(&lang, \"назад\", \"back\")}" }
                }
                h1 { class: "text-3xl font-bold", "{i18n::tr(&lang, \"Создание аккаунта\", \"Create account\")}" }
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

            PasswordField {
                value: confirm,
                placeholder: i18n::tr(&lang, "подтвердите пароль", "confirm password"),
                class: "input input-bordered max-w-md".to_string(),
            }

            button {
                class: "btn btn-primary btn-sm gap-1",
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
                {icon_element(Icon::Plus, 16)}
                span { "{i18n::tr(&lang, \"создать аккаунт\", \"create account\")}" }
            }
        }
    }
}

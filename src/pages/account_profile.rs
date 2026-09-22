use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    api,
    components::{
        admin_badge::AdminBadge,
        datetime_text::DateTimeText,
        delete_form::DeleteForm,
        icon::{Icon, icon_element},
    },
    i18n,
    models::DeletionRequest,
    state::STATE,
};

#[component]
pub fn Account() -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let mut deleting = use_signal(|| false);
    let user = STATE.read().user.clone();

    let Some(user) = user else {
        return rsx! {
            div { class: "flex flex-col items-start gap-4 max-w-7xl mx-auto w-full",
                button { class: "btn btn-ghost btn-sm gap-2", onclick: move |_| { let _ = navigator.push(crate::Route::Home {}); },                     {icon_element(Icon::Back, 16)}, span { "{i18n::tr(&lang, \"назад\", \"back\")}" } }
                p { class: "italic", "{i18n::tr(&lang, \"Вы не вошли в аккаунт\", \"You are not logged in\")}" }
            }
        };
    };

    let level = user.admin_level.clone();

    rsx! {
        div { class: "flex flex-col items-start gap-4 max-w-7xl mx-auto w-full",
            h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Профиль\", \"Profile\")}" }

            div { class: "card w-full max-w-3xl bg-base-200 shadow-lg overflow-hidden",
                div { class: match &level {
                    crate::models::users::AdminLevel::User => "h-24 bg-gradient-to-r from-info to-primary",
                    crate::models::users::AdminLevel::Admin => "h-24 bg-gradient-to-r from-secondary to-accent",
                    crate::models::users::AdminLevel::Owner => "h-24 bg-gradient-to-r from-warning to-error",
                } }
                div { class: "card-body gap-3 pt-0",
                    div { class: "-mt-10 rounded-full w-20 h-20 flex items-center justify-center bg-neutral text-neutral-content text-2xl font-bold ring-4 ring-base-200",
                        "{user.login.chars().next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default()}"
                    }
                    div { class: "flex flex-wrap items-center gap-2",
                        h2 { class: "card-title text-2xl", "{user.login}" }
                        AdminBadge { level: level.clone(), lang: lang.clone() }
                    }
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-sm", "id" }
                        span { class: "text-sm italic font-semibold", "{user.id}" }
                    }
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-sm", "{i18n::tr(&lang, \"аккаунт создан\", \"account created\")}" }
                        DateTimeText { time: user.created_at, class: "text-sm italic font-semibold text-right" }
                    }
                }
            }

            button {
                class: "btn btn-sm gap-1",
                onclick: {
                    move |_| {
                        STATE.write().user = None;
                        STATE.write().token = None;
                        crate::state::clear_token();
                        navigator.push(crate::Route::Welcome {});
                    }
                },
                {icon_element(Icon::Exit, 16)}
                span { "{i18n::tr(&lang, \"выйти\", \"logout\")}" }
            }

            button {
                class: "btn btn-error btn-sm gap-1",
                onclick: move |_| deleting.set(!deleting()),
                {icon_element(Icon::Trash, 16)}
                span { "{i18n::tr(&lang, \"удалить аккаунт\", \"delete account\")}" }
            }

            if deleting() {
                DeleteForm {
                    on_delete: move |(login, password, confirm)| {
                        let token = STATE.read().token.clone();
                        let request = DeletionRequest { login, password, deletion_confirmation: confirm };
                        let navigator = navigator;
                        spawn(async move {
                            match api::users::delete_my_account(&request, &token).await {
                                Ok(()) => {
                                    STATE.write().user = None;
                                    STATE.write().token = None;
                                    crate::state::clear_token();
                                    show_alert(AlertKind::Success, i18n::tr(&crate::state::language(), "Аккаунт удалён", "Account deleted"));
                                    navigator.push(crate::Route::Welcome {});
                                }
                                Err(e) => show_alert(AlertKind::Error, e),
                            }
                        });
                    },
                    on_cancel: move |_| deleting.set(false),
                }
            }
        }
    }
}

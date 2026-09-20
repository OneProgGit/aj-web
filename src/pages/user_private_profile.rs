use dioxus::prelude::*;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        admin_badge::AdminBadge,
        datetime_text::DateTimeText,
        delete_form::DeleteForm,
        icon::{icon_element, Icon},
    },
    i18n,
    models::{users::AdminLevel, DeletionRequest},
    state::STATE,
};

#[component]
pub fn UserPrivateProfile(user_id: i64) -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let mut loaded = use_signal(|| false);
    let mut user = use_signal(|| None::<crate::models::users::PrivateUserData>);
    let mut level_idx = use_signal(|| 0usize);
    let mut deleting = use_signal(|| false);
    let mut busy = use_signal(|| false);

    if !*loaded.read() {
        loaded.set(true);
        let token = crate::state::token();
        spawn(async move {
            match api::users::get_private_user(user_id, &token).await {
                Ok(fetched) => {
                    level_idx.set(match fetched.admin_level {
                        AdminLevel::User => 0,
                        AdminLevel::Admin => 1,
                        AdminLevel::Owner => 2,
                    });
                    user.set(Some(fetched));
                }
                Err(e) => show_alert(AlertKind::Error, e),
            }
        });
    }

    let profile_data = user().map(|p| (p.admin_level.clone(), p));

    rsx! {
        div { class: "flex flex-col items-start gap-4 max-w-7xl mx-auto w-full",
            button {
                class: "btn btn-ghost btn-sm gap-2",
                onclick: move |_| {
                    navigator.go_back();
                },
                {icon_element(Icon::Back, 16)}
                span { "{i18n::tr(&lang, \"назад\", \"back\")}" }
            }

            h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Профиль\", \"Profile\")}" }

            if let Some((level, profile)) = profile_data {
                div { class: "card w-full max-w-3xl bg-base-200 shadow-lg overflow-hidden",
                    div { class: match &level {
                        crate::models::users::AdminLevel::User => "h-24 bg-gradient-to-r from-info to-primary",
                        crate::models::users::AdminLevel::Admin => "h-24 bg-gradient-to-r from-secondary to-accent",
                        crate::models::users::AdminLevel::Owner => "h-24 bg-gradient-to-r from-warning to-error",
                    } }
                    div { class: "card-body gap-3 pt-0",
                        div { class: "-mt-10 rounded-full w-20 h-20 flex items-center justify-center bg-neutral text-neutral-content text-2xl font-bold ring-4 ring-base-200",
                            "{profile.login.chars().next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default()}"
                        }
                        div { class: "flex flex-wrap items-center gap-2",
                            h2 { class: "card-title text-2xl", "{profile.login}" }
                            AdminBadge { level: level.clone(), lang: lang.clone() }
                        }
                        div { class: "flex items-center justify-between gap-4",
                            span { class: "text-sm", "id" }
                            span { class: "text-sm italic font-semibold", "{profile.id}" }
                        }
                        div { class: "flex items-center justify-between gap-4",
                            span { class: "text-sm", "{i18n::tr(&lang, \"аккаунт создан\", \"account created\")}" }
                            DateTimeText { time: profile.created_at, class: "text-sm italic font-semibold text-right" }
                        }
                    }
                }

                div { class: "card w-full max-w-3xl bg-base-200 shadow-lg",
                    div { class: "card-body gap-3",
                        h3 { class: "card-title text-base", "{i18n::tr(&lang, \"Уровень админа\", \"Admin level\")}" }
                        div { class: "flex flex-wrap gap-4 items-center",
                            select {
                        class: "select select-bordered select-sm",
                        value: level_idx().to_string(),
                        onchange: move |ev| level_idx.set(ev.value().parse::<usize>().unwrap_or(0)),
                        option { value: "0", "{i18n::tr(&lang, \"пользователь\", \"user\")}" }
                        option { value: "1", "{i18n::tr(&lang, \"админ\", \"admin\")}" }
                        option { value: "2", "{i18n::tr(&lang, \"владелец\", \"owner\")}" }
                    }
                    button {
                        class: "btn btn-primary btn-sm gap-1",
                        disabled: busy(),
                        onclick: {
                            let token = STATE.read().token.clone();
                            let user_id = user_id;
                            move |_| {
                                if busy() {
                                    return;
                                }
                                busy.set(true);
                                let level = match level_idx() {
                                    0 => AdminLevel::User,
                                    1 => AdminLevel::Admin,
                                    _ => AdminLevel::Owner,
                                };
                                let token = token.clone();
                                spawn(async move {
                                    match api::users::update_admin_level(user_id, &level, &token).await {
                                        Ok(()) => {
                                            busy.set(false);
                                            show_alert(AlertKind::Success, i18n::tr(&crate::state::language(), "Уровень админа обновлён", "Admin level updated"));
                                            if let Some(mut u) = user() {
                                                u.admin_level = level.clone();
                                                user.set(Some(u));
                                            }
                                            if let Some(me) = STATE.read().user.clone() {
                                                if me.id == user_id {
                                                    if let Some(u) = STATE.write().user.as_mut() {
                                                        u.admin_level = level;
                                                    }
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
                        {icon_element(Icon::Pencil, 16)}
                        span { "{i18n::tr(&lang, \"изменить\", \"edit\")}" }
                    }
                    }
                    }
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
                            let nav = navigator;
                            spawn(async move {
                                match api::users::delete_user_account(user_id, &request, &token).await {
                                    Ok(()) => {
                                        deleting.set(false);
                                        show_alert(AlertKind::Success, i18n::tr(&crate::state::language(), "Аккаунт удалён", "Account deleted"));
                                        if STATE.read().user.as_ref().is_some_and(|me| me.id == user_id) {
                                            STATE.write().user = None;
                                            STATE.write().token = None;
                                            crate::state::clear_token();
                                            nav.push(crate::Route::Welcome {});
                                        }
                                    }
                                    Err(e) => show_alert(AlertKind::Error, e),
                                }
                            });
                        },
                        on_cancel: move |_| deleting.set(false),
                    }
                }
            } else {
                p { class: "italic", "{i18n::tr(&lang, \"Нет данных\", \"No data\")}" }
            }
        }
    }
}

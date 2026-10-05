use dioxus::prelude::*;

use crate::{
    alerts::show_alert,
    api,
    components::{
        admin_badge::AdminBadge,
        icon::{Icon, icon_element},
        loading::Loading,
        users_ws::{unsubscribe_user_ws, user_ws},
    },
    i18n,
};

#[component]
pub fn UserProfile(user_id: i64) -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    // Загрузка привязана к user_id: при переходе между профилями компонент
    // не пересоздаётся, а один spawn на профиль (иначе цикл ререндеров).
    let mut started_for = use_signal(|| None::<i64>);
    let mut loaded = use_signal(|| false);
    let mut user = use_signal(|| None::<crate::models::users::PublicUserData>);

    if started_for() != Some(user_id) {
        started_for.set(Some(user_id));
        loaded.set(false);
        user.set(None);
        // Живое обновление профиля: лента шлёт событие при смене данных,
        // страница перечитывает свой сигнал — как у ленты контестов, где
        // событие молча обновляет состояние без тостов.
        let mut live_user = user;
        user_ws(user_id, move |_event| async move {
            let token = crate::state::token();
            if let Ok(fetched) = api::users::get_public_user(user_id, &token).await {
                live_user.set(Some(fetched));
            }
        });
        let token = crate::state::token();
        spawn(async move {
            let res = api::users::get_public_user(user_id, &token).await;
            match res {
                Ok(fetched) => user.set(Some(fetched)),
                Err(e) => show_alert(crate::alerts::AlertKind::Error, e),
            }
            loaded.set(true);
        });
    }

    use_drop(move || unsubscribe_user_ws(user_id));

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

            if !loaded() {
                Loading {}
            } else if let Some((level, profile)) = profile_data {
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
                    }
                }
            } else {
                p { class: "italic", "{i18n::tr(&lang, \"Нет данных\", \"No data\")}" }
            }
        }
    }
}

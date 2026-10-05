#![allow(non_snake_case)]

mod alerts;
mod api;
mod components;
mod i18n;
mod models;
mod pages;
mod state;

use dioxus::prelude::*;
use dioxus::router::Link;

use crate::{
    alerts::AlertHost,
    components::users_ws::user_ws,
    pages::{
        account_profile::Account, contest::Contest, home::Home, login::Login, problems::Problems,
        register::Register, user_private_profile::UserPrivateProfile, user_profile::UserProfile,
        users::Users, welcome::Welcome,
    },
    state::STATE,
};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[layout(GuardLayout)]
    #[route("/")]
    Welcome {},
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/home")]
    Home {},
    #[route("/contest/:contest_id")]
    Contest { contest_id: i64 },
    #[route("/problems")]
    Problems {},
    #[route("/users")]
    Users {},
    #[route("/account")]
    Account {},
    #[route("/user/:user_id")]
    UserProfile { user_id: i64 },
    #[route("/user/:user_id/private")]
    UserPrivateProfile { user_id: i64 },
}

/// Routes that require an authenticated user.
fn is_guarded(route: &Route) -> bool {
    matches!(
        route,
        Route::Home { .. }
            | Route::Contest { .. }
            | Route::Problems { .. }
            | Route::Users { .. }
            | Route::Account { .. }
            | Route::UserProfile { .. }
            | Route::UserPrivateProfile { .. }
    )
}

/// Layout wrapping every route. Runs the auth guard (redirects away from
/// guarded routes when logged out and away from Welcome when logged in).
/// Must live inside `Router`, so it is a layout rather than part of `App`.
#[component]
fn GuardLayout() -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let location = use_route::<Route>();
    let logged_in = STATE.read().token.is_some();

    let title = match &location {
        Route::Welcome {} => i18n::tr(&lang, "Добро пожаловать — aj-web", "Welcome — aj-web"),
        Route::Login {} => i18n::tr(&lang, "Вход — aj-web", "Login — aj-web"),
        Route::Register {} => i18n::tr(&lang, "Регистрация — aj-web", "Register — aj-web"),
        Route::Home {} => i18n::tr(&lang, "Контесты — aj-web", "Contests — aj-web"),
        Route::Contest { contest_id } => i18n::tr(
            &lang,
            &format!("Контест #{contest_id} — aj-web"),
            &format!("Contest #{contest_id} — aj-web"),
        ),
        Route::Problems {} => i18n::tr(&lang, "Задачи — aj-web", "Problems — aj-web"),
        Route::Users {} => i18n::tr(&lang, "Пользователи — aj-web", "Users — aj-web"),
        Route::Account {} | Route::UserProfile { .. } | Route::UserPrivateProfile { .. } => {
            i18n::tr(&lang, "Профиль — aj-web", "Profile — aj-web")
        }
    };

    use_effect(use_reactive(
        &(location.clone(), logged_in),
        move |(route, logged_in)| {
            if is_guarded(&route) && !logged_in {
                let _ = navigator.replace(Route::Welcome {});
            } else if matches!(route, Route::Welcome {}) && logged_in {
                let _ = navigator.replace(Route::Home {});
            }
        },
    ));

    rsx! {
        document::Title { "{title}" }
        document::Stylesheet { href: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css" }
        document::Stylesheet { href: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css", media: "(prefers-color-scheme: dark)" }
        document::Script { src: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js" }
        div { class: "min-h-dvh flex flex-col",
        nav { class: "navbar bg-base-100 border-b border-base-300",
            div { class: "flex items-center gap-4 px-3 sm:px-6 max-w-7xl mx-auto w-full",
                Link { to: "/", class: "btn btn-ghost btn-sm font-bold", "ada-judge" }
                if logged_in {
                    div { class: "tabs tabs-box max-w-full overflow-x-auto",
                        Link {
                            to: Route::Home {},
                            class: if matches!(&location, Route::Home {} | Route::Contest { .. }) { "tab gap-2 tab-active" } else { "tab gap-2" },
                            {crate::components::icon::icon_element(crate::components::icon::Icon::Home, 16)}
                            span { "{i18n::tr(&lang, \"главная\", \"home\")}" }
                        }
                        if STATE.read().is_admin() {
                            Link {
                                to: Route::Problems {},
                                class: if matches!(&location, Route::Problems {}) { "tab gap-2 tab-active" } else { "tab gap-2" },
                                {crate::components::icon::icon_element(crate::components::icon::Icon::Reader, 16)}
                                span { "{i18n::tr(&lang, \"задачи\", \"problems\")}" }
                            }
                        }
                        if STATE.read().is_owner() {
                            Link {
                                to: Route::Users {},
                                class: if matches!(&location, Route::Users {}) { "tab gap-2 tab-active" } else { "tab gap-2" },
                                {crate::components::icon::icon_element(crate::components::icon::Icon::Person, 16)}
                                span { "{i18n::tr(&lang, \"пользователи\", \"users\")}" }
                            }
                        }
                        Link {
                            to: Route::Account {},
                            class: if matches!(&location, Route::Account {}) { "tab gap-2 tab-active" } else { "tab gap-2" },
                            {crate::components::icon::icon_element(crate::components::icon::Icon::Account, 16)}
                            span { "{i18n::tr(&lang, \"профиль\", \"profile\")}" }
                        }
                    }
                }
                div { class: "flex-1" }
                // Переключатель языка — кнопками, как фильтры статусов:
                // вариантов всего два, текущий виден сразу, переключение
                // в одно нажатие. Названия языков не переводим.
                div { class: "flex items-center gap-1 rounded-lg border border-base-300 bg-base-200 p-0.5",
                    for (code, title) in [("ru", "русский"), ("en", "english")] {
                        button {
                            class: if lang == *code {
                                "btn btn-xs btn-primary"
                            } else {
                                "btn btn-xs btn-ghost"
                            },
                            onclick: {
                                let code = code.to_string();
                                move |_| {
                                    STATE.write().language = code.clone();
                                    crate::state::save_language(&code);
                                }
                            },
                            "{title}"
                        }
                    }
                }
            }
        }
        div { class: "max-w-7xl mx-auto p-3 sm:p-6 w-full flex-1",
            Outlet::<Route> {}
        }
        footer { class: "footer footer-center p-4 text-base-content/70 text-sm sticky bottom-0 z-10 bg-base-100 border-t border-base-300",
            aside { p { "© 2026 OneProg" } }
        }
        }
    }
}

pub fn App() -> Element {
    let mut loaded_user = use_signal(|| false);

    // Подписка на свой профиль: уровень админа может сменить другой админ,
    // и тогда вкладки навбара должны появиться/пропасть без перезагрузки.
    // По событию перечитываем /users/me — STATE.user обновится, навбар
    // перерисуется сам, потому что читает is_admin()/is_owner() из него.
    fn watch_own_profile(user_id: i64) {
        user_ws(user_id, move |_event| async move {
            if let Ok(me) = crate::api::users::get_me(&crate::state::token()).await {
                STATE.write().user = Some(me);
            }
        });
    }

    // За кем следим сейчас: id может появиться позже (логин без перезагрузки)
    // или смениться (выход и вход другим пользователем). Сравнение на каждом
    // рендере вместо once-флага — иначе подписка, стартовавшая до логина,
    // никогда не откроется. Старую подписку закрываем, чтобы не копить.
    let mut watched_user = use_signal(|| None::<i64>);
    let current_user = STATE.read().user.as_ref().map(|u| u.id);
    if watched_user() != current_user {
        if let Some(old) = watched_user() {
            crate::components::users_ws::unsubscribe_user_ws(old);
        }
        watched_user.set(current_user);
        if let Some(id) = current_user {
            watch_own_profile(id);
        }
    }

    // On a fresh page load (e.g. after F5 or a full navigation) the user
    // profile is only in memory, so re-fetch /users/me when a token exists.
    // Подписка выше стартует сама, как только STATE.user появится.
    if !*loaded_user.read() {
        loaded_user.set(true);
        if STATE.read().token.is_some() && STATE.read().user.is_none() {
            let token = STATE.read().token.clone();
            spawn(async move {
                if let Ok(me) = crate::api::users::get_me(&token).await {
                    STATE.write().user = Some(me);
                }
            });
        }
    }

    // Статус контеста выводится из времени, но условия и разбор приходят только
    // с сервера — за ними нужно следить на любой странице, а не только там,
    // где открыт контест.
    crate::components::contest_ws::contest_status_watcher();

    rsx! {
        document::Stylesheet { href: asset!("/public/tailwind.css") }
        div { class: "min-h-screen bg-base-100",
            Router::<Route> {}
        }
        AlertHost {}
    }
}

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
    components::users_ws::{own_profile_ws, unsubscribe_own_profile_ws},
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
        Route::Welcome {} => i18n::tr(&lang, "Добро пожаловать — ada-judge", "Welcome — ada-judge"),
        Route::Login {} => i18n::tr(&lang, "Вход — ada-judge", "Login — ada-judge"),
        Route::Register {} => i18n::tr(&lang, "Регистрация — ada-judge", "Register — ada-judge"),
        Route::Home {} => i18n::tr(&lang, "Контесты — ada-judge", "Contests — ada-judge"),
        Route::Contest { contest_id } => i18n::tr(
            &lang,
            &format!("Контест #{contest_id} — ada-judge"),
            &format!("Contest #{contest_id} — ada-judge"),
        ),
        Route::Problems {} => i18n::tr(&lang, "Задачи — ada-judge", "Problems — ada-judge"),
        Route::Users {} => i18n::tr(&lang, "Пользователи — ada-judge", "Users — ada-judge"),
        Route::Account {} | Route::UserProfile { .. } | Route::UserPrivateProfile { .. } => {
            i18n::tr(&lang, "Профиль — ada-judge", "Profile — ada-judge")
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
            // На мобиле два ряда: сверху логотип + переключатель языка,
            // под ними вкладки горизонтально (со скроллом). На sm+ всё
            // в одну строку, как раньше.
            div { class: "flex flex-wrap items-center gap-2 px-3 sm:gap-4 sm:px-6 max-w-7xl mx-auto w-full",
                // Логотип — не кнопка: просто текст-ссылка, как в oneprog-cup.
                Link { to: "/", class: "font-bold px-2 py-1 text-base-content hover:text-primary", "ada-judge" }
                if logged_in {
                    // order-last + w-full: на мобиле вкладки уходят вторым
                    // рядом на всю ширину; на sm+ возвращаются в общий ряд.
                    div { class: "tabs tabs-box max-w-full overflow-x-auto order-last w-full sm:order-none sm:w-auto",
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
                // ml-auto: на мобиле переключатель прижат вправо в первом
                // ряду рядом с логотипом; на sm+ правее его толкает тот же margin.
                // Отдельный flex-1 больше не нужен.
                // Переключатель языка — кнопками, как фильтры статусов:
                // вариантов всего два, текущий виден сразу, переключение
                // в одно нажатие. Названия языков не переводим.
                div { class: "ml-auto flex items-center gap-1 rounded-lg border border-base-300 bg-base-200 p-0.5",
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
    fn watch_own_profile() {
        // Подписка одна на приложение (ключ "user:me"), id не нужен: сервер
        // сам понимает, чей профиль слать, по токену из query.
        own_profile_ws(move |_event| async move {
            if let Ok(me) = crate::api::users::get_me(&crate::state::token()).await {
                // Тост синий: уровень сменил кто-то другой, а вкладки сейчас
                // перерисуются — без пояснения это выглядит как глюк.
                let lang = crate::state::language();
                let before = STATE.read().user.as_ref().map(|u| u.admin_level.clone());
                let changed = before.as_ref().is_some_and(|old| *old != me.admin_level);
                STATE.write().user = Some(me);
                if changed {
                    crate::alerts::show_alert(
                        crate::alerts::AlertKind::Info,
                        crate::i18n::tr(
                            &lang,
                            "Ваш уровень доступа изменён",
                            "Your access level changed",
                        ),
                    );
                }
            }
        });
    }

    // За кем следим сейчас: id может появиться позже (логин без перезагрузки)
    // или смениться (выход и вход другим пользователем). Сравнение на каждом
    // рендере вместо once-флага — иначе подписка, стартовавшая до логина,
    // никогда не откроется. Старую подписку закрываем, чтобы не копить.
    // Подписка живёт пока залогинены: при выходе закрываем, при входе
    // открываем. user_id здесь не нужен — сервер определяет «свой» сам.
    let mut watching_own = use_signal(|| false);
    let logged_in = STATE.read().user.is_some();
    if watching_own() != logged_in {
        watching_own.set(logged_in);
        if logged_in {
            watch_own_profile();
        } else {
            unsubscribe_own_profile_ws();
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

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
    components::select::M3Select,
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
        // M3E грузится из MaterialHost локальным файлом /m3e.js.
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
                M3Select {
                    id: "aj-ui-lang",
                    label: i18n::tr(&lang, "язык", "language"),
                    options: vec![
                        ("ru".to_string(), i18n::tr(&lang, "русский", "russian")),
                        ("en".to_string(), "english".to_string()),
                    ],
                    value: lang.clone(),
                    onchange: Callback::new(|value: String| {
                        STATE.write().language = value.clone();
                        crate::state::save_language(&value);
                    }),
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

    // On a fresh page load (e.g. after F5 or a full navigation) the user
    // profile is only in memory, so re-fetch /users/me when a token exists.
    if !*loaded_user.read() {
        loaded_user.set(true);
        let has_token = STATE.read().token.is_some();
        let user_missing = STATE.read().user.is_none();
        if has_token && user_missing {
            spawn(async move {
                if let Ok(me) = crate::api::users::get_me(&crate::state::token()).await {
                    STATE.write().user = Some(me);
                }
            });
        }
    }

    rsx! {
        document::Stylesheet { href: asset!("/public/tailwind.css") }
        // m3e-theme раздаёт компонентам их собственные токены (--md-sys-color-*):
        // без него цвета не вычисляются и элементы выглядят «пустыми».
        // Палитра — та же, что у daisyUI-темы: Material You из #4d256e.
        m3e-theme {
            color: "#4d256e",
            variant: "tonal-spot",
            scheme: "auto",
            motion: "expressive",
            div { class: "min-h-dvh flex flex-col",
                Router::<Route> {}
            }
            AlertHost {}
            MaterialHost {}
        }
    }
}

/// Загрузка M3E и проверка, что компоненты зарегистрированы.
///
/// Динамический `import()` внутри `eval` требует `unsafe-eval` в CSP,
/// а его обычно нет — поэтому вставляем обычный
/// `<script type="module" src="/m3e.js">` через DOM — локальный файл
/// собирается esbuild'ом (npm run build:m3e), CDN и сеть не нужны.
#[component]
pub fn MaterialHost() -> Element {
    let failed = use_signal(|| false);
    let mut failed2 = failed;
    use_effect(move || {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let Ok(el) = document.create_element("script") else {
            return;
        };
        // Скрипт с src выполняется при вставке — в отличие от инлайнового.
        el.set_attribute("type", "module").ok();
        el.set_attribute("src", "/m3e.js").ok();
        document.head().and_then(|h| h.append_child(&el).ok());

        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            let ok = js_sys::eval("(() => !!customElements.get('m3e-button'))()")
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !ok {
                failed2.set(true);
            }
        });
    });
    rsx! {
        if failed() {
            div { class: "alert alert-error m-3 text-sm",
                "M3E не загрузился (/m3e.js) — выполни npm run build:m3e"
            }
        }
    }
}

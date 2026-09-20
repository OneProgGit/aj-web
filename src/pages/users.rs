use dioxus::prelude::*;

use crate::{alerts::show_alert, api, components::user_card::UserCard, i18n, state::STATE};

#[component]
pub fn Users() -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let mut loaded = use_signal(|| false);

    if !*loaded.read() {
        loaded.set(true);
        let token = crate::state::token();
        spawn(async move {
            match api::users::get_all_users(&token).await {
                Ok(list) => STATE.write().users = list,
                Err(e) => show_alert(crate::alerts::AlertKind::Error, e),
            }
        });
    }

    rsx! {
        div { class: "flex flex-col gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex flex-wrap gap-4 items-center",
                button {
                    class: "btn btn-ghost btn-sm gap-2",
                    onclick: move |_| { let _ = navigator.push(crate::Route::Home {}); },
                    {crate::components::icon::icon_element(crate::components::icon::Icon::Back, 16)}
                    span { "{i18n::tr(&lang, \"назад\", \"back\")}" }
                }
                h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Пользователи\", \"Users\")}" }
            }

            if STATE.read().users.is_empty() {
                p { class: "italic", "{i18n::tr(&lang, \"Пользователей пока что нет\", \"No users yet\")}" }
            } else {
                div { class: "flex flex-col gap-4 w-full max-h-[32rem] overflow-y-auto",
                    for u in STATE.read().users.iter() {
                        UserCard { user: u.clone() }
                    }
                }
            }
        }
    }
}

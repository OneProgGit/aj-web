use dioxus::prelude::*;

use crate::{
    alerts::show_alert,
    api,
    components::{loading::Loading, user_card::UserCard},
    i18n,
    state::STATE,
};

#[component]
pub fn Users() -> Element {
    let lang = crate::state::language();
    let mut started = use_signal(|| false);
    let mut loading = use_signal(|| false);

    if !started() {
        started.set(true);
        loading.set(true);
        let token = crate::state::token();
        spawn(async move {
            let res = api::users::get_all_users(&token).await;
            match res {
                Ok(list) => STATE.write().users = list,
                Err(e) => show_alert(crate::alerts::AlertKind::Error, e),
            }
            loading.set(false);
        });
    }

    rsx! {
        div { class: "flex flex-col gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex flex-wrap gap-4 items-center",
                h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Пользователи\", \"Users\")}" }
            }

            if loading() {
                Loading {}
            } else if STATE.read().users.is_empty() {
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

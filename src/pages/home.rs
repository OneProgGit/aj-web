use dioxus::prelude::*;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        contest_card::ContestCard,
        contest_form::contest_form,
        contest_ws::contests_feed_ws,
        icon::Icon,
    },
    i18n,
    models::contests::ContestRequest,
    state::STATE,
};

/// Loads account/contest/problem/user data, mirroring aj-app's `load_all`.
async fn reload_data(
    all_contests: bool,
    force_contests: bool,
    force_problems: bool,
    force_users: bool,
) {
    let token = crate::state::token();
    if all_contests && force_contests {
        match api::contests::get_contests(&token).await {
            Ok(list) => {
                STATE.write().contests = list;
                contests_feed_ws(false);
            }
            Err(e) => show_alert(AlertKind::Error, e),
        }
    }
    if !all_contests && force_contests {
        match api::contests::get_my_contests(&token).await {
            Ok(list) => {
                STATE.write().contests = list;
                contests_feed_ws(true);
            }
            Err(e) => show_alert(AlertKind::Error, e),
        }
    }
    if force_problems {
        let is_admin = STATE.read().is_admin();
        if is_admin {
            match api::problems::get_all_problems(&token).await {
                Ok(list) => STATE.write().problems = list,
                Err(e) => show_alert(AlertKind::Error, e),
            }
        }
    }
    if force_users && STATE.read().is_owner() {
        match api::users::get_all_users(&token).await {
            Ok(list) => STATE.write().users = list,
            Err(e) => show_alert(AlertKind::Error, e),
        }
    }
}

#[component]
pub fn Home() -> Element {
    let lang = crate::state::language();
    let mut modal_open = use_signal(|| false);
    let mut loaded = use_signal(|| false);

    if !*loaded.read() {
        loaded.set(true);
        spawn(async move {
            let all = STATE.read().contests_is_all;
            reload_data(all, true, true, true).await;
        });
    }

    rsx! {
        div { class: "flex flex-col items-start gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex gap-4 items-center",
                h2 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Контесты\", \"Contests\")}" }

                if STATE.read().is_admin() {
                    button {
                        class: "btn btn-neutral btn-sm gap-1",
                        onclick: move |_| modal_open.set(true),
                        {crate::components::icon::icon_element(Icon::Plus, 16)}
                        span { "{i18n::tr(&lang, \"создать\", \"create\")}" }
                    }
                }
            }

            if STATE.read().is_admin() {
                label { class: "label cursor-pointer justify-start gap-2",
                input {
                    r#type: "checkbox",
                    class: "checkbox checkbox-sm",
                    checked: STATE.read().contests_is_all,
                    onchange: move |ev| {
                        let new_value = ev.checked();
                        STATE.write().contests_is_all = new_value;
                        let token = crate::state::token();
                        spawn(async move {
                            if new_value {
                                match api::contests::get_contests(&token).await {
                                    Ok(list) => {
                                        STATE.write().contests = list;
                                    }
                                    Err(e) => show_alert(AlertKind::Error, e),
                                }
                            } else {
                                match api::contests::get_my_contests(&token).await {
                                    Ok(list) => {
                                        STATE.write().contests = list;
                                    }
                                    Err(e) => show_alert(AlertKind::Error, e),
                                }
                            }
                        });
                    },
                }
                span { class: "label-text", "{i18n::tr(&lang, \"все контесты\", \"all contests\")}" }
            }
            }

            if STATE.read().contests.is_empty() {
                p { class: "italic", "{i18n::tr(&lang, \"Контестов пока что нет\", \"No contests yet\")}" }
            } else {
                div { class: "flex flex-col gap-4 w-full max-h-[28rem] overflow-y-auto",
                    for contest in STATE.read().contests.iter() {
                        ContestCard {
                            contest: contest.clone(),
                            show_enter: true,
                            compact: false,
                            on_changed: move |_| {
                                let all = STATE.read().contests_is_all;
                                spawn(async move {
                                    reload_data(all, true, false, false).await;
                                });
                            },
                        }
                    }
                }
            }

            if modal_open() {
                div { class: "modal modal-open",
                    div { class: "modal-box max-w-2xl",
                        div { class: "flex items-center justify-between",
                            h3 { class: "card-title", "{i18n::tr(&lang, \"Создать контест\", \"Create contest\")}" }
                            button {
                                class: "btn btn-sm btn-circle btn-ghost",
                                onclick: move |_| modal_open.set(false),
                                "✕",
                            }
                        }
                        {contest_form(
                            None,
                            &i18n::tr(&lang, "создать", "create"),
                            Callback::new(move |request: ContestRequest| {
                                let token = crate::state::token();
                                spawn(async move {
                                    match api::contests::create_contest(&request, &token).await {
                                        Ok(()) => {
                                            modal_open.set(false);
                                        }
                                        Err(e) => show_alert(AlertKind::Error, e),
                                    }
                                });
                            }),
                            Callback::new(move |_: MouseEvent| modal_open.set(false)),
                        )}
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    api,
    components::{
        contest_card::ContestCard, contest_form::contest_form, contest_ws::contests_feed_ws,
        icon::Icon, loading::Loading,
    },
    i18n,
    models::contests::ContestRequest,
    state::{ContestStatus, STATE, contest_status},
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
    // started — запрос пошёл (ровно один раз), loading — он ещё идёт.
    let mut started = use_signal(|| false);
    let mut loading = use_signal(|| false);
    // Фильтр по статусу: None — показывать все, как отдаёт сервер.
    let status_filter = use_signal(|| None::<ContestStatus>);

    if !started() {
        started.set(true);
        loading.set(true);
        spawn(async move {
            let all = STATE.read().contests_is_all;
            reload_data(all, true, true, true).await;
            loading.set(false);
        });
    }

    // Статус считаем здесь, один раз на контест: карточка сама пересчитает
    // его на каждом тике, а список перерисовывается вместе с ней.
    let visible_contests: Vec<_> = STATE
        .read()
        .contests
        .iter()
        .filter(|c| match status_filter() {
            Some(want) => contest_status(c) == want,
            None => true,
        })
        .cloned()
        .collect();
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
                    class: "toggle toggle-sm",
                    checked: STATE.read().contests_is_all,
                    // Пока идёт перезагрузка списка, переключатель гасим:
                    // иначе можно дёрнуть его второй раз и получить
                    // два конкурирующих запроса — какой ответ придёт последним,
                    // тот и победит, а значение тумблера может не совпасть
                    // с показанным списком.
                    disabled: loading(),
                    onchange: move |ev| {
                        let new_value = ev.checked();
                        STATE.write().contests_is_all = new_value;
                        let token = crate::state::token();
                        loading.set(true);
                        spawn(async move {
                            let res = if new_value {
                                api::contests::get_contests(&token).await
                            } else {
                                api::contests::get_my_contests(&token).await
                            };
                            loading.set(false);
                            match res {
                                Ok(list) => STATE.write().contests = list,
                                Err(e) => show_alert(AlertKind::Error, e),
                            }
                        });
                    },
                }
                span { class: "label-text", "{i18n::tr(&lang, \"все контесты\", \"all contests\")}" }
            }
            }

            if loading() {
                Loading {}
            } else if STATE.read().contests.is_empty() {
                p { class: "italic", "{i18n::tr(&lang, \"Контестов пока что нет\", \"No contests yet\")}" }
            } else {
                div { class: "flex flex-wrap items-center gap-2",
                    // Порядок серверный (id desc) — сортировки нет, только фильтр.
                    for option in [
                        None,
                        Some(ContestStatus::Ongoing),
                        Some(ContestStatus::BeforeStart),
                        Some(ContestStatus::Upsolving),
                        Some(ContestStatus::Finished),
                    ] {
                        {
                            let mut filter = status_filter;
                            let active = status_filter() == option;
                            let title = match option {
                                None => i18n::tr(&lang, "все", "all"),
                                Some(st) => st.label(&lang),
                            };
                            rsx! {
                                button {
                                    class: if active {
                                        "btn btn-sm btn-primary"
                                    } else {
                                        "btn btn-sm btn-ghost"
                                    },
                                    onclick: move |_| filter.set(option),
                                    "{title}"
                                }
                            }
                        }
                    }
                }
                div { class: "flex flex-col gap-4 w-full max-h-[28rem] overflow-y-auto",
                    // Сюда попадаем только когда сам список непуст, так что
                    // пустой результат — это именно следствие фильтра.
                    if visible_contests.is_empty() {
                        p { class: "italic",
                            "{i18n::tr(&lang, \"По этому фильтру ничего нет\", \"Nothing matches this filter\")}"
                        }
                    }
                    for contest in visible_contests.iter() {
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

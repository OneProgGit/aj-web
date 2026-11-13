use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    api,
    components::{
        contest_card::ContestCard,
        contest_form::{ContestForm, ContestInit},
        contest_ws::contests_feed_ws,
        icon::Icon,
        loading::Loading,
    },
    i18n,
    models::contests::ContestRequest,
    query::{ContestFilter, HomeQuery},
    state::{STATE, contest_status},
};

/// Loads contest/user data, mirroring aj-app's `load_all`.
///
/// Задачи здесь не грузим: главная их всё равно не рисует (список живёт на
/// `Route::Problems`, где данные тянет сама страница), а `GET /problems`
/// отдаётся только владельцу — запрос от обычного админа возвращал 403.
async fn reload_data(all_contests: bool, force_contests: bool, force_users: bool) {
    let token = crate::state::token();
    if all_contests && force_contests {
        match api::contests::get_contests(&token).await {
            Ok(list) => {
                STATE.write().contests = list;
                contests_feed_ws();
            }
            Err(e) => crate::state::show_error(e),
        }
    }
    if !all_contests && force_contests {
        match api::contests::get_my_contests(&token).await {
            Ok(list) => {
                STATE.write().contests = list;
                contests_feed_ws();
            }
            Err(e) => crate::state::show_error(e),
        }
    }
    if force_users && STATE.read().is_owner() {
        match api::users::get_all_users(&token).await {
            Ok(list) => STATE.write().users = list,
            Err(e) => crate::state::show_error(e),
        }
    }
}

#[component]
pub fn Home(query: HomeQuery) -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let mut modal_open = use_signal(|| false);
    // loading — запрос ещё идёт; loaded_for — для какого all он уже выполнен.
    let mut loading = use_signal(|| false);
    let mut loaded_for = use_signal(|| None::<bool>);

    if loaded_for() != Some(query.all) {
        loaded_for.set(Some(query.all));
        loading.set(true);
        spawn(async move {
            reload_data(query.all, true, true).await;
            loading.set(false);
        });
    }

    // Статус считаем здесь, один раз на контест: карточка сама пересчитает
    // его на каждом тике, а список перерисовывается вместе с ней.
    let visible_contests: Vec<_> = STATE
        .read()
        .contests
        .iter()
        .filter(|c| match query.status.status() {
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
                        checked: query.all,
                        disabled: loading(),
                        onchange: move |ev| {
                            let _ = navigator.replace(crate::Route::Home {
                                query: HomeQuery {
                                    all: ev.checked(),
                                    status: query.status,
                                },
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
                        ContestFilter::All,
                        ContestFilter::Ongoing,
                        ContestFilter::BeforeStart,
                        ContestFilter::Upsolving,
                        ContestFilter::Finished,
                    ] {
                        {
                            let active = query.status == option;
                            let title = match option.status() {
                                None => i18n::tr(&lang, "все", "all"),
                                Some(status) => status.label(&lang),
                            };
                            rsx! {
                                button {
                                    class: if active {
                                        "btn btn-sm btn-primary"
                                    } else {
                                        "btn btn-sm btn-ghost"
                                    },
                                    onclick: move |_| {
                                        let _ = navigator.replace(crate::Route::Home {
                                            query: HomeQuery {
                                                all: query.all,
                                                status: option,
                                            },
                                        });
                                    },
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
                                let all = query.all;
                                spawn(async move {
                                    reload_data(all, true, false).await;
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
                        ContestForm {
                            initial: ContestInit(None),
                            submit_label: i18n::tr(&lang, "создать", "create"),
                            on_submit: Callback::new(move |request: ContestRequest| {
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
                            on_cancel: Callback::new(move |_: MouseEvent| modal_open.set(false)),
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;

use crate::{
    api, i18n,
    models::{DeletionRequest, contests::PublicContestConfig},
    state::{ContestStatus, STATE, contest_status},
};

use super::{
    contest_form::contest_form,
    datetime_text::DateTimeText,
    delete_form::DeleteForm,
    icon::{Icon, icon_element},
};

/// Total duration in human units (`3 дн 2 ч`), no seconds.
fn format_duration(lang: &str, dur: chrono::Duration) -> String {
    let m = dur.num_minutes().max(0);
    let (d, h, mm) = (m / 1440, m % 1440 / 60, m % 60);
    let (du, hu, mu) = if lang == "en" {
        ("d", "h", "min")
    } else {
        ("дн", "ч", "мин")
    };
    if d > 0 {
        format!("{d} {du} {h} {hu} {mm} {mu}")
    } else if h > 0 {
        format!("{h} {hu} {mm} {mu}")
    } else {
        format!("{mm} {mu}")
    }
}

/// Remaining time in human units (`1 дн 2 ч 14 мин`), same visual style
/// as the start/end dates.
fn format_remaining(lang: &str, left: chrono::Duration) -> String {
    let s = left.num_seconds().max(0);
    let (d, h, m, sec) = (s / 86400, s % 86400 / 3600, s % 3600 / 60, s % 60);
    let (du, hu, mu, su) = if lang == "en" {
        ("d", "h", "min", "s")
    } else {
        ("дн", "ч", "мин", "с")
    };
    if d > 0 {
        format!("{d} {du} {h} {hu} {m} {mu} {sec} {su}")
    } else if h > 0 {
        format!("{h} {hu} {m} {mu} {sec} {su}")
    } else if m > 0 {
        format!("{m} {mu} {sec} {su}")
    } else {
        format!("{sec} {su}")
    }
}

fn yes_no(lang: &str, value: bool) -> String {
    if value {
        i18n::tr(lang, "да", "yes")
    } else {
        i18n::tr(lang, "нет", "no")
    }
}
/// whether the «войти» button is rendered (contest page sets it to false).
#[derive(Props, Clone)]
/// Contest card. Mirrors `AJContestCard` from aj-app. `show_enter` controls
/// whether the «войти» button is rendered (contest page sets it to false).
pub struct ContestCardProps {
    pub contest: PublicContestConfig,
    pub show_enter: bool,
    pub on_changed: EventHandler<()>,
    pub compact: bool,
}

impl PartialEq for ContestCardProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.contest, &other.contest)
            && self.show_enter == other.show_enter
            && self.on_changed == other.on_changed
            && self.compact == other.compact
    }
}

pub fn ContestCard(props: ContestCardProps) -> Element {
    let ContestCardProps {
        contest,
        show_enter,
        on_changed,
        compact,
    } = props;
    let lang = crate::state::language();
    let navigator = use_navigator();
    let status = contest_status(&contest);
    let can_manage = STATE.read().can_manage_contest(&contest);
    let can_enter = STATE.read().can_enter_contest(&contest);

    let mut editing = use_signal(|| false);
    let mut deleting = use_signal(|| false);
    let mut info_open = use_signal(|| false);
    let mut now = use_signal(chrono::Utc::now);
    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(1000).await;
                now.set(chrono::Utc::now());
            }
        });
    });

    let name = if lang == "en" && !contest.name_en.is_empty() {
        contest.name_en.clone()
    } else {
        contest.name_ru.clone()
    };
    let has_collapsed = contest.owner_id.is_some();
    let owner_login = contest.owner_login.clone().unwrap_or_default();

    let has_url = !contest.statements_url_ru.is_empty()
        || !contest.editorial_url_ru.is_empty()
        || !contest.statements_url_en.is_empty()
        || !contest.editorial_url_en.is_empty();

    let statements_ru = contest.statements_url_ru.clone();
    let editorial_ru = contest.editorial_url_ru.clone();
    let statements_en = contest.statements_url_en.clone();
    let editorial_en = contest.editorial_url_en.clone();

    rsx! {
        div { class: "card w-full max-w-3xl shadow-lg {status.css_bg()} {status.css_text()}",
            div { class: "card-body gap-3",
                div { class: "flex items-center justify-between gap-4",
                    h2 { class: "card-title text-lg",
                        span { "#{contest.id} {name}" }
                    }
                    span { class: "text-sm italic",
                        match status {
                            ContestStatus::BeforeStart => i18n::tr(&lang, "не начался", "has not started"),
                            ContestStatus::Ongoing => i18n::tr(&lang, "идёт", "ongoing"),
                            ContestStatus::Finished => i18n::tr(&lang, "завершён", "finished"),
                            ContestStatus::Upsolving => i18n::tr(&lang, "дорешка", "upsolving"),
                        }
                    }
                }

                if has_url {
                    div { class: "flex flex-wrap gap-2",
                        if lang == "ru" && !contest.statements_url_ru.is_empty() {
                            button {
                                class: "btn btn-neutral btn-sm gap-1",
                                onclick: move |_| crate::state::open_link(&statements_ru),
                                {icon_element(Icon::FileText, 14)}
                                span { "{i18n::tr(&lang, \"условия задач\", \"statements\")}" }
                            }
                        }
                        if lang == "ru" && !contest.editorial_url_ru.is_empty() {
                            button {
                                class: "btn btn-neutral btn-sm gap-1",
                                onclick: move |_| crate::state::open_link(&editorial_ru),
                                {icon_element(Icon::FileText, 14)}
                                span { "{i18n::tr(&lang, \"разбор задач\", \"editorial\")}" }
                            }
                        }
                        if lang == "en" && !contest.statements_url_en.is_empty() {
                            button {
                                class: "btn btn-neutral btn-sm gap-1",
                                onclick: move |_| crate::state::open_link(&statements_en),
                                {icon_element(Icon::FileText, 14)}
                                span { "{i18n::tr(&lang, \"условия задач\", \"statements\")}" }
                            }
                        }
                        if lang == "en" && !contest.editorial_url_en.is_empty() {
                            button {
                                class: "btn btn-neutral btn-sm gap-1",
                                onclick: move |_| crate::state::open_link(&editorial_en),
                                {icon_element(Icon::FileText, 14)}
                                span { "{i18n::tr(&lang, \"разбор задач\", \"editorial\")}" }
                            }
                        }
                    }
                }

                if has_collapsed {
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-sm", "{i18n::tr(&lang, \"владелец\", \"owner\")}" }
                        button {
                            class: "btn btn-neutral btn-sm gap-1",
                            onclick: {
                                let owner_id = contest.owner_id.unwrap_or_default();
                                let is_owner = STATE.read().is_owner();
                                move |_| {
                                    if is_owner {
                                        let _ = navigator.push(crate::Route::UserPrivateProfile { user_id: owner_id });
                                    } else {
                                        let _ = navigator.push(crate::Route::UserProfile { user_id: owner_id });
                                    }
                                }
                            },
                            {icon_element(Icon::Person, 14)}
                            span { "{owner_login}" }
                        }
                    }
                }

                if status == ContestStatus::Ongoing {
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-sm", "{i18n::tr(&lang, \"начало\", \"start\")}" }
                        DateTimeText { time: contest.starts_at, class: "text-sm italic font-semibold text-right" }
                    }
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-sm", "{i18n::tr(&lang, \"осталось\", \"remaining\")}" }
                        span { class: "text-sm italic font-semibold",
                            {format_remaining(&lang, contest.finishes_at - now())}
                        }
                    }
                } else if !compact {
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-sm", "{i18n::tr(&lang, \"начало\", \"start\")}" }
                        DateTimeText { time: contest.starts_at, class: "text-sm italic font-semibold text-right" }
                    }
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-sm", "{i18n::tr(&lang, \"продолжительность\", \"duration\")}" }
                        span { class: "text-sm italic font-semibold",
                            {format_duration(&lang, contest.finishes_at - contest.starts_at)}
                        }
                    }
                } else {
                    div { class: "flex items-center justify-between gap-4",
                        span { class: "text-sm", "{i18n::tr(&lang, \"продолжительность\", \"duration\")}" }
                        span { class: "text-sm italic font-semibold",
                            {format_duration(&lang, contest.finishes_at - contest.starts_at)}
                        }
                    }
                }

                div { class: "flex flex-wrap items-center gap-2",
                    if (status != ContestStatus::BeforeStart || can_enter) && show_enter {
                        button {
                            class: "btn btn-neutral btn-sm gap-1",
                            onclick: {
                                move |_| { let _ = navigator.push(crate::Route::Contest { contest_id: contest.id }); }
                            },
                            {icon_element(Icon::Enter, 16)}
                            span { "{i18n::tr(&lang, \"войти\", \"enter\")}" }
                        }
                    }
                    if can_manage {
                        button {
                            class: "btn btn-neutral btn-sm gap-1",
                            onclick: move |_| editing.set(!editing()),
                            {icon_element(Icon::Pencil, 16)}
                            span { "{i18n::tr(&lang, \"изменить\", \"edit\")}" }
                        }
                        button {
                            class: "btn btn-neutral btn-sm gap-1",
                            onclick: move |_| deleting.set(!deleting()),
                            {icon_element(Icon::Trash, 16)}
                            span { "{i18n::tr(&lang, \"удалить\", \"delete\")}" }
                        }
                    } else {
                        button {
                            class: "btn btn-neutral btn-sm gap-1",
                            onclick: move |_| info_open.set(!info_open()),
                            {icon_element(Icon::Info, 16)}
                            span { "{i18n::tr(&lang, \"инфо\", \"info\")}" }
                        }
                    }
                }

                if editing() {
                    div { class: "modal modal-open",
                        div { class: "modal-box max-w-2xl text-base-content",
                            div { class: "flex items-center justify-between",
                                h3 { class: "card-title", "{i18n::tr(&lang, \"Изменить контест\", \"Edit contest\")}" }
                                button {
                                    class: "btn btn-sm btn-circle btn-ghost",
                                    onclick: move |_| editing.set(false),
                                    "✕"
                                }
                            }
                            {contest_form(
                                Some(&contest),
                                &i18n::tr(&lang, "изменить", "edit"),
                                Callback::new(move |request| {
                                    let token = STATE.read().token.clone();
                                    let id = contest.id;
                                    spawn(async move {
                                        match api::contests::update_contest(id, &request, &token).await {
                                            Ok(()) => {
                                                editing.set(false);
                                                on_changed.call(());
                                            }
                                            Err(e) => crate::alerts::show_alert(crate::alerts::AlertKind::Error, e),
                                        }
                                    });
                                }),
                                Callback::new(move |_| editing.set(false)),
                            )}
                        }
                    }
                }

                if info_open() {
                    div { class: "modal modal-open",
                        div { class: "modal-box max-w-2xl text-base-content",
                            div { class: "flex items-center justify-between",
                                h3 { class: "card-title", "{i18n::tr(&lang, \"Информация о контесте\", \"Contest info\")}" }
                                button {
                                    class: "btn btn-sm btn-circle btn-ghost",
                                    onclick: move |_| info_open.set(false),
                                    "✕"
                                }
                            }
                            div { class: "flex flex-col gap-3 mt-4",
                                div { class: "flex items-center justify-between gap-4",
                                    span { class: "text-sm", "{i18n::tr(&lang, \"дорешка\", \"upsolving\")}" }
                                    span { class: "text-sm italic font-semibold text-right", "{yes_no(&lang, contest.upsolving_enabled)}" }
                                }
                                div { class: "flex items-center justify-between gap-4",
                                    span { class: "text-sm", "{i18n::tr(&lang, \"решения скрыты\", \"solutions hidden\")}" }
                                    span { class: "text-sm italic font-semibold text-right", "{yes_no(&lang, contest.solutions_hidden)}" }
                                }
                                div { class: "flex items-center justify-between gap-4",
                                    span { class: "text-sm", "{i18n::tr(&lang, \"таблица скрыта\", \"leaderboard hidden\")}" }
                                    span { class: "text-sm italic font-semibold text-right", "{yes_no(&lang, contest.leaderboard_hidden)}" }
                                }
                            }
                        }
                    }
                }

                if deleting() {
                    DeleteForm {
                            on_delete: move |(login, password, confirm)| {
                                let token = STATE.read().token.clone();
                                let id = contest.id;
                                let request = DeletionRequest { login, password, deletion_confirmation: confirm };
                                spawn(async move {
                                    match api::contests::delete_contest(id, &request, &token).await {
                                        Ok(()) => {
                                            deleting.set(false);
                                            on_changed.call(());
                                        }
                                        Err(e) => crate::alerts::show_alert(crate::alerts::AlertKind::Error, e),
                                    }
                                });
                            },
                            on_cancel: move |_| deleting.set(false),
                        }
                }
            }
        }
    }
}

use chrono::{Duration, Utc};
use dioxus::prelude::*;

use crate::{
    i18n,
    models::contests::{ContestRequest, PublicContestConfig},
};

use super::datetime_input::datetime_input;
use super::icon::{icon_element, Icon};

/// Create/edit form for a contest. Mirrors the «создать контест» and
/// «изменить контест» popups from aj-app's HomePage / ContestCard.
pub fn contest_form(
    initial: Option<&PublicContestConfig>,
    submit_label: &str,
    on_submit: EventHandler<ContestRequest>,
    on_cancel: EventHandler<MouseEvent>,
) -> Element {
    let lang = crate::state::language();
    let now = Utc::now();

    let mut name_ru = use_signal(|| initial.map_or(String::new(), |c| c.name_ru.clone()));
    let mut name_en = use_signal(|| initial.map_or(String::new(), |c| c.name_en.clone()));
    let mut starts = use_signal(|| initial.map_or(now, |c| c.starts_at));
    let init_secs = initial.map_or(7200, |c| {
        (c.finishes_at - c.starts_at).num_seconds().max(0)
    });
    let mut dur_d = use_signal(|| (init_secs / 86400).to_string());
    let mut dur_h = use_signal(|| ((init_secs % 86400) / 3600).to_string());
    let mut dur_m = use_signal(|| ((init_secs % 3600) / 60).to_string());
    let mut dur_s = use_signal(|| (init_secs % 60).to_string());
    let duration_secs = move || {
        let p = |v: String| v.parse::<i64>().unwrap_or(0).max(0);
        p(dur_d()) * 86400 + p(dur_h()) * 3600 + p(dur_m()) * 60 + p(dur_s())
    };
    let mut s_url_ru = use_signal(|| initial.map_or(String::new(), |c| c.statements_url_ru.clone()));
    let mut e_url_ru = use_signal(|| initial.map_or(String::new(), |c| c.editorial_url_ru.clone()));
    let mut s_url_en = use_signal(|| initial.map_or(String::new(), |c| c.statements_url_en.clone()));
    let mut e_url_en = use_signal(|| initial.map_or(String::new(), |c| c.editorial_url_en.clone()));
    let mut hidden = use_signal(|| initial.map(|c| c.hidden).unwrap_or(false));
    let mut upsolving = use_signal(|| initial.map(|c| c.upsolving_enabled).unwrap_or(false));
    let mut hide_solutions = use_signal(|| initial.map(|c| c.solutions_hidden).unwrap_or(false));
    let mut hide_leaderboard = use_signal(|| initial.map(|c| c.leaderboard_hidden).unwrap_or(false));
    let mut co_authors = use_signal(|| {
        initial.map_or(String::new(), |c| {
            c.co_authors
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        })
    });

    let valid = !name_ru().is_empty() || !name_en().is_empty();
    let invalid = !valid || duration_secs() <= 0;

    rsx! {
        div { class: "flex flex-col gap-3",
            span { class: "label-text", "{i18n::tr(&lang, \"название (рус.)\", \"name (ru)\")}" }
            input {
                class: "input input-bordered",
                value: name_ru(),
                oninput: move |ev| name_ru.set(ev.value()),
            }
            span { class: "label-text", "{i18n::tr(&lang, \"название (англ.)\", \"name (en)\")}" }
            input {
                class: "input input-bordered",
                value: name_en(),
                oninput: move |ev| name_en.set(ev.value()),
            }
                span { class: "label-text", "{i18n::tr(&lang, \"начало в\", \"starts at\")}" }
                {datetime_input(starts(), Callback::new(move |dt| starts.set(dt)))}
                span { class: "label-text", "{i18n::tr(&lang, \"продолжительность\", \"duration\")}" }
                div { class: "flex flex-wrap items-center gap-2",
                    input {
                        class: "input input-bordered w-20",
                        r#type: "number",
                        min: "0",
                        value: dur_d(),
                        oninput: move |ev| dur_d.set(ev.value()),
                    }
                    span { class: "text-sm", "{i18n::tr(&lang, \"дн\", \"d\")}" }
                    input {
                        class: "input input-bordered w-20",
                        r#type: "number",
                        min: "0",
                        value: dur_h(),
                        oninput: move |ev| dur_h.set(ev.value()),
                    }
                    span { class: "text-sm", "{i18n::tr(&lang, \"ч\", \"h\")}" }
                    input {
                        class: "input input-bordered w-20",
                        r#type: "number",
                        min: "0",
                        value: dur_m(),
                        oninput: move |ev| dur_m.set(ev.value()),
                    }
                    span { class: "text-sm", "{i18n::tr(&lang, \"мин\", \"min\")}" }
                    input {
                        class: "input input-bordered w-20",
                        r#type: "number",
                        min: "0",
                        value: dur_s(),
                        oninput: move |ev| dur_s.set(ev.value()),
                    }
                    span { class: "text-sm", "{i18n::tr(&lang, \"с\", \"s\")}" }
                }
                span { class: "label-text", "{i18n::tr(&lang, \"ссылка на условия (рус.)\", \"statements url (ru)\")}" }
                input {
                    class: "input input-bordered",
                    value: s_url_ru(),
                    oninput: move |ev| s_url_ru.set(ev.value()),
                }
                span { class: "label-text", "{i18n::tr(&lang, \"ссылка на разбор (рус.)\", \"editorial url (ru)\")}" }
                input {
                    class: "input input-bordered",
                    value: e_url_ru(),
                    oninput: move |ev| e_url_ru.set(ev.value()),
                }
                span { class: "label-text", "{i18n::tr(&lang, \"ссылка на условия (англ.)\", \"statements url (en)\")}" }
                input {
                    class: "input input-bordered",
                    value: s_url_en(),
                    oninput: move |ev| s_url_en.set(ev.value()),
                }
                span { class: "label-text", "{i18n::tr(&lang, \"ссылка на разбор (англ.)\", \"editorial url (en)\")}" }
                input {
                    class: "input input-bordered",
                    value: e_url_en(),
                    oninput: move |ev| e_url_en.set(ev.value()),
                }
                label { class: "label cursor-pointer justify-start gap-2",
                    input { r#type: "checkbox", class: "checkbox checkbox-sm", checked: hidden(), onchange: move |ev| hidden.set(ev.checked()) }
                    span { class: "label-text", "{i18n::tr(&lang, \"скрыть\", \"hidden\")}" }
                }
                label { class: "label cursor-pointer justify-start gap-2",
                    input { r#type: "checkbox", class: "checkbox checkbox-sm", checked: upsolving(), onchange: move |ev| upsolving.set(ev.checked()) }
                    span { class: "label-text", "{i18n::tr(&lang, \"открыть дорешку\", \"open upsolving\")}" }
                }
                label { class: "label cursor-pointer justify-start gap-2",
                    input { r#type: "checkbox", class: "checkbox checkbox-sm", checked: hide_solutions(), onchange: move |ev| hide_solutions.set(ev.checked()) }
                    span { class: "label-text", "{i18n::tr(&lang, \"скрыть решения\", \"hide solutions\")}" }
                }
                label { class: "label cursor-pointer justify-start gap-2",
                    input { r#type: "checkbox", class: "checkbox checkbox-sm", checked: hide_leaderboard(), onchange: move |ev| hide_leaderboard.set(ev.checked()) }
                    span { class: "label-text", "{i18n::tr(&lang, \"скрыть таблицу лидеров\", \"hide leaderboard\")}" }
                }
                span { class: "label-text", "{i18n::tr(&lang, \"соавторы (через запятую, без пробелов)\", \"co-authors (comma-separated)\")}" }
                input {
                    class: "input input-bordered",
                    value: co_authors(),
                    oninput: move |ev| co_authors.set(ev.value()),
                }
                div { class: "card-actions justify-end mt-2",
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: move |ev| on_cancel.call(ev),
                        span { "{i18n::tr(&lang, \"отменить\", \"cancel\")}" }
                    }
                    button {
                        class: if initial.is_some() { "btn btn-primary btn-sm gap-2" } else { "btn btn-neutral btn-sm gap-1" },
                        disabled: invalid,
                        onclick: move |_| {
                            let request = ContestRequest {
                                name_ru: name_ru(),
                                name_en: name_en(),
                                starts_at: starts(),
                                finishes_at: starts() + Duration::seconds(duration_secs()),
                                statements_url_ru: s_url_ru(),
                                editorial_url_ru: e_url_ru(),
                                statements_url_en: s_url_en(),
                                editorial_url_en: e_url_en(),
                                hidden: hidden(),
                                upsolving_enabled: upsolving(),
                                solutions_hidden: hide_solutions(),
                                leaderboard_hidden: hide_leaderboard(),
                                co_authors: co_authors()
                                    .split(',')
                                    .filter_map(|s| s.trim().parse().ok())
                                    .collect(),
                            };
                            on_submit.call(request);
                        },
                        {icon_element(if valid { if initial.is_some() { Icon::Pencil } else { Icon::Plus } } else { Icon::CircleBackslash }, 16)}
                        span { "{submit_label}" }
                    }
                }
        }
    }
}

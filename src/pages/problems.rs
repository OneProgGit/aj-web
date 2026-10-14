use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    api,
    components::{loading::Loading, problem_card::ProblemCard, problems_ws::problems_feed_ws},
    i18n,
    state::STATE,
};

#[component]
pub fn Problems() -> Element {
    let lang = crate::state::language();
    let mut all_problems = use_signal(|| false);
    let mut started = use_signal(|| false);
    let mut loading = use_signal(|| false);
    // Имя контеста по id: показываем его в карточке каждой задачи.
    let mut contest_names = use_signal(HashMap::<i64, String>::new);
    let is_owner = STATE.read().is_owner();

    if !started() {
        started.set(true);
        loading.set(true);
        STATE.write().problems = Vec::new();
        let all = all_problems();
        let token = crate::state::token();
        // lang уходит в задачу-спавн, рендеру оставляем свою копию.
        let names_lang = lang.clone();
        spawn(async move {
            // Названия контестов тянем отдельным запросом: у задачи есть только
            // contest_id, а показывать надо и имя. Ошибку здесь глушим — список
            // задач от неё зависеть не должен, в худшем случае останется id.
            if let Ok(cs) = api::contests::get_contests(&token).await {
                contest_names.set(
                    cs.iter()
                        .map(|c| {
                            (
                                c.id,
                                i18n::contest_title(&names_lang, &c.name_ru, &c.name_en),
                            )
                        })
                        .collect(),
                );
            }
            let res = if all {
                api::problems::get_all_problems(&token).await
            } else {
                api::problems::get_my_problems(&token).await
            };
            match res {
                Ok(list) => {
                    STATE.write().problems = list;
                    problems_feed_ws();
                }
                Err(e) => show_alert(AlertKind::Error, e),
            }
            loading.set(false);
        });
    }

    rsx! {
        div { class: "flex flex-col gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex flex-wrap gap-4 items-center",
                h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Задачи\", \"Problems\")}" }
            }

            if is_owner {
                label { class: "label cursor-pointer justify-start gap-2",
                    input {
                        r#type: "checkbox",
                        class: "toggle toggle-sm",
                        checked: all_problems(),
                        // Гасим на время запроса — иначе два конкурирующих
                        // запроса решат, какой список останется, и значение
                        // тумблера может разойтись с показанным списком.
                        disabled: loading(),
                        onchange: move |ev| {
                            let new_value = ev.checked();
                            all_problems.set(new_value);
                            let token = crate::state::token();
                            loading.set(true);
                            spawn(async move {
                                let res = if new_value {
                                    api::problems::get_all_problems(&token).await
                                } else {
                                    api::problems::get_my_problems(&token).await
                                };
                                loading.set(false);
                                match res {
                                    Ok(list) => {
                                        STATE.write().problems = list;
                                        problems_feed_ws();
                                    }
                                    Err(e) => show_alert(AlertKind::Error, e),
                                }
                            });
                        },
                    }
                    span { class: "label-text", "{i18n::tr(&lang, \"все задачи\", \"all problems\")}" }
                }
            }

            if loading() {
                Loading {}
            } else if STATE.read().problems.is_empty() {
                p { class: "italic", "{i18n::tr(&lang, \"Задач пока что нет\", \"No problems yet\")}" }
            } else {
                div { class: "flex flex-col gap-4 w-full max-h-[32rem] overflow-y-auto",
                    for p in STATE.read().problems.iter() {
                        div { class: "flex flex-col gap-1",
                            // Задача принадлежит контесту, но на списке это не
                            // видно — показываем имя и id ссылкой на него.
                            div { class: "flex items-center gap-2 px-1 text-sm opacity-70",
                                Link {
                                    to: "/contest/{p.contest_id}",
                                    class: "link link-hover",
                                    "{contest_names().get(&p.contest_id).cloned().unwrap_or_else(|| i18n::tr(&lang, \"контест\", \"contest\"))}",
                                }
                                span { class: "badge badge-sm badge-ghost", "#{p.contest_id}" }
                            }
                            ProblemCard {
                                problem: p.clone(),
                                show_id: true,
                                subgroups_scroll: true,
                                on_changed: move |_| {},
                            }
                        }
                    }
                }
            }
        }
    }
}

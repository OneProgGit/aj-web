use dioxus::prelude::*;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        icon::{icon_element, Icon},
        markdown::MdField,
        question_card::QuestionCard,
    },
    i18n,
    models::problems::ProblemQuestionRequest,
    state::STATE,
};

#[component]
pub fn ContestQuestions(contest_id: i64) -> Element {
    let lang = crate::state::language();
    let mut modal_open = use_signal(|| false);
    let mut problem_idx = use_signal(|| 0usize);
    let mut title = use_signal(String::new);
    let text = use_signal(String::new);

    let problems = STATE.read().contest_problems.clone();
    let questions = STATE.read().questions.clone();
    let user_id = STATE.read().user.as_ref().map(|u| u.id);
    let is_owner = STATE.read().is_owner();
    let contest_owned = STATE
        .read()
        .contests
        .iter()
        .find(|c| c.id == contest_id)
        .is_some_and(|c| STATE.read().can_manage_contest(c));

    let valid = !title().is_empty() && !text().is_empty();

    let can_manage = |q: &crate::models::problems::ProblemQuestion| {
        if contest_owned || is_owner {
            return true;
        }
        problems
            .iter()
            .find(|p| p.id == q.problem_id)
            .is_some_and(|p| STATE.read().can_manage_problem(p))
    };

    let question_rows = questions
        .iter()
        .cloned()
        .map(|q| {
            let problem_name = problems.iter().find(|p| p.id == q.problem_id).map_or_else(
                || "?".to_string(),
                |p| format!("#{} {}", p.index + 1, i18n::problem_title(&lang, &p.name_ru, &p.name_en)),
            );
            let can_answer = can_manage(&q);
            let can_delete = user_id.is_some_and(|id| id == q.owner_id)
                || is_owner
                || contest_owned
                || problems
                    .iter()
                    .find(|p| p.id == q.problem_id)
                    .is_some_and(|p| STATE.read().can_manage_problem(p));
            (q, problem_name, can_answer, can_delete)
        })
        .collect::<Vec<_>>();

    rsx! {
        div { class: "flex flex-col gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex flex-wrap gap-4 items-center",
                h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Вопросы по задачам\", \"Problem questions\")}" }

                button {
                    class: "btn btn-neutral btn-sm gap-1",
                    onclick: move |_| modal_open.set(true),
                    {icon_element(Icon::QuestionMark, 16)}
                    span { "{i18n::tr(&lang, \"задать вопрос\", \"ask question\")}" }
                }
            }

            if questions.is_empty() {
                p { class: "italic", "{i18n::tr(&lang, \"Вопросов пока что нет\", \"No questions yet\")}" }
            } else {
                div { class: "flex flex-col gap-4 w-full max-h-[32rem] overflow-y-auto",
                    for (i, (q, problem_name, can_answer, can_delete)) in question_rows.iter().enumerate() {
                        QuestionCard {
                            question: q.clone(),
                            position: question_rows.len() - i,
                            problem_name: problem_name.clone(),
                            can_answer: *can_answer,
                            can_delete: *can_delete,
                            on_changed: {
                                move |_| {}
                            },
                        }
                    }
                }
            }

            if modal_open() {
                div { class: "modal modal-open",
                    div { class: "modal-box max-w-2xl",
                        div { class: "flex items-center justify-between",
                            h3 { class: "card-title", "{i18n::tr(&lang, \"Задать вопрос\", \"Ask a question\")}" }
                            button { class: "btn btn-sm btn-circle btn-ghost", onclick: move |_| modal_open.set(false), "✕" }
                        }
                        div { class: "flex flex-col gap-3 mt-4",
                            span { class: "label-text", "{i18n::tr(&lang, \"задача\", \"problem\")}" }
                            select {
                                class: "select select-bordered select-sm",
                                value: problem_idx().to_string(),
                                onchange: move |ev| problem_idx.set(ev.value().parse::<usize>().unwrap_or(0)),
                                for (i, p) in problems.iter().enumerate() {
                                    option { value: "{i}", "#{p.index + 1} {i18n::problem_title(&lang, &p.name_ru, &p.name_en)}" }
                                }
                            }
                            span { class: "label-text", "{i18n::tr(&lang, \"название\", \"title\")}" }
                            input {
                                class: "input input-bordered",
                                value: title(),
                                oninput: move |ev| title.set(ev.value()),
                            }
                            MdField {
                                value: text,
                                label: i18n::tr(&lang, "текст", "text"),
                            }
                            div { class: "card-actions justify-end",
                                button {
                                    class: "btn btn-ghost btn-sm gap-1",
                                    onclick: move |_| modal_open.set(false),
                                    span { "{i18n::tr(&lang, \"отменить\", \"cancel\")}" }
                                }
                                button {
                                    class: "btn btn-primary btn-sm gap-1",
                                    disabled: !valid || problems.is_empty(),
                                    onclick: {
                                        move |_| {
                                            let Some(problem) = problems.get(problem_idx()).cloned() else { return; };
                                            let request = ProblemQuestionRequest { title: title(), text: text() };
                                            let token = crate::state::token();
                                            let mut modal = modal_open;
                                            spawn(async move {
                                                match api::contests::create_problem_question(problem.id, &request, &token).await {
                                                    Ok(()) => {
                                                        modal.set(false);
                                                    }
                                                    Err(e) => show_alert(AlertKind::Error, e),
                                                }
                                            });
                                        }
                                    },
                                    {icon_element(if valid { Icon::Plus } else { Icon::CircleBackslash }, 16)}
                                    span { "{i18n::tr(&lang, \"задать вопрос\", \"ask question\")}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

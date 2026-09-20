use dioxus::prelude::*;

use crate::{
    api, i18n,
    models::{problems::ProblemQuestion, DeletionRequest},
    state::STATE,
};

use super::{
    datetime_text::DateTimeText,
    delete_form::DeleteForm,
    icon::{icon_element, Icon},
    markdown::{Markdown, MdField},
    user_link::UserLink,
};

fn answer_form(
    question: ProblemQuestion,
    on_done: EventHandler<()>,
    on_cancel: Callback<MouseEvent>,
) -> Element {
    let lang = crate::state::language();
    let answer = use_signal(|| question.answer.clone());
    let valid = !answer().is_empty();
    rsx! {
        div { class: "flex flex-col gap-3",
        MdField {
            value: answer,
            label: i18n::tr(&lang, "ответ", "answer"),
        }
        div { class: "card-actions justify-end",
            button {
                class: "btn btn-ghost btn-sm gap-1",
                onclick: move |ev| on_cancel.call(ev),
                span { "{i18n::tr(&lang, \"отменить\", \"cancel\")}" }
            }
            button {
                class: "btn btn-primary btn-sm gap-1",
                disabled: !valid,
                onclick: move |_| {
                    let token = STATE.read().token.clone();
                    let qid = question.id;
                    let text = answer();
                    spawn(async move {
                        match api::contests::answer_problem_question(qid, &text, &token).await {
                            Ok(()) => {
                                on_done.call(());
                            }
                            Err(e) => crate::alerts::show_alert(crate::alerts::AlertKind::Error, e),
                        }
                    });
                },
                {icon_element(if valid { Icon::Pencil } else { Icon::CircleBackslash }, 16)}
                span { "{i18n::tr(&lang, \"ответить\", \"answer\")}" }
            }
        }
        }
    }
}

/// Question card. Mirrors `AJProblemQuestionCard`.
#[derive(Props, Clone)]
pub struct QuestionCardProps {
    pub question: ProblemQuestion,
    pub problem_name: String,
    pub can_answer: bool,
    pub can_delete: bool,
    pub on_changed: EventHandler<()>,
}

impl PartialEq for QuestionCardProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.question, &other.question)
            && self.problem_name == other.problem_name
            && self.can_answer == other.can_answer
            && self.can_delete == other.can_delete
            && self.on_changed == other.on_changed
    }
}

pub fn QuestionCard(props: QuestionCardProps) -> Element {
    let QuestionCardProps {
        question,
        problem_name,
        can_answer,
        can_delete,
        on_changed,
    } = props;
    let lang = crate::state::language();
    let mut answering = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    rsx! {
        div { class: "card w-full max-w-3xl bg-base-200 shadow-lg",
            div { class: "card-body gap-3",
                div { class: "flex items-center justify-between gap-4",
                    span { class: "text-sm", "{i18n::tr(&lang, \"автор\", \"author\")}" }
                    UserLink { user_id: question.owner_id, username: question.owner_login.clone() }
                }
                div { class: "flex items-center justify-between gap-4",
                    span { class: "text-sm", "{i18n::tr(&lang, \"задача\", \"problem\")}" }
                    span { class: "text-sm font-medium text-right", "{problem_name}" }
                }
                div { class: "flex items-center justify-between gap-4",
                    span { class: "text-sm", "{i18n::tr(&lang, \"создан\", \"created\")}" }
                    DateTimeText { time: question.created_at, class: "text-sm italic text-right" }
                }
                h3 { class: "card-title", span { "#{question.index + 1} {question.title}" } }
                Markdown { text: question.text.clone() }
                if !question.answer.is_empty() {
                    h3 { class: "card-title", span { "{i18n::tr(&lang, \"Ответ\", \"Answer\")}" } }
                    Markdown { text: question.answer.clone() }
                }

                div { class: "flex items-center gap-2",
                    if can_answer {
                        button {
                            class: "btn btn-ghost btn-sm gap-1",
                            onclick: move |_| answering.set(!answering()),
                            {icon_element(Icon::Pencil, 16)}
                            span { "{i18n::tr(&lang, \"ответить\", \"answer\")}" }
                        }
                    }
                    if can_delete {
                        button {
                            class: "btn btn-ghost btn-sm gap-1 text-error",
                            onclick: move |_| deleting.set(!deleting()),
                            {icon_element(Icon::Trash, 16)}
                            span { "{i18n::tr(&lang, \"удалить\", \"delete\")}" }
                        }
                    }
                }

                if answering() {
                    div { class: "modal modal-open",
                        div { class: "modal-box max-w-2xl",
                            div { class: "flex items-center justify-between",
                                h3 { class: "card-title", "{i18n::tr(&lang, \"Ответ\", \"Answer\")}" }
                                button {
                                    class: "btn btn-sm btn-circle btn-ghost",
                                    onclick: move |_| answering.set(false),
                                    "✕"
                                }
                            }
                            {answer_form(question.clone(), Callback::new(move |_| {
                                answering.set(false);
                                on_changed.call(());
                            }), Callback::new(move |_| answering.set(false)))}
                        }
                    }
                }

                if deleting() {
                    DeleteForm {
                        on_delete: move |(login, password, confirm)| {
                            let token = STATE.read().token.clone();
                            let qid = question.id;
                            let request = DeletionRequest { login, password, deletion_confirmation: confirm };
                            spawn(async move {
                                match api::contests::delete_problem_question(qid, &request, &token).await {
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

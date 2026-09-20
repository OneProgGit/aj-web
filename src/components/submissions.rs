use dioxus::prelude::*;

use crate::{
    api, i18n,
    models::{problems::PublicProblemConfig, testing::Submission, verdicts::TestingVerdict},
    state::STATE,
};

use super::{
    datetime_text::DateTimeText,
    icon::{icon_element, Icon},
    markdown::Markdown,
    user_link::UserLink,
    verdict_badge::VerdictBadge,
};

/// «Посылки» table. Mirrors `AJSubmissions`. Clicking «подробнее» opens a
/// modal with subgroup/test verdicts (web replacement for AJSubmissionCard).
#[derive(Props, Clone)]
pub struct SubmissionsProps {
    pub submissions: Vec<Submission>,
    pub problem: PublicProblemConfig,
    pub contest_hide_solutions: bool,
    pub contest_can_manage: bool,
    pub all_submissions_allowed: bool,
    pub on_changed: EventHandler<()>,
}

impl PartialEq for SubmissionsProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.submissions, &other.submissions)
            && crate::models::props_json_eq(&self.problem, &other.problem)
            && self.contest_hide_solutions == other.contest_hide_solutions
            && self.contest_can_manage == other.contest_can_manage
            && self.all_submissions_allowed == other.all_submissions_allowed
            && self.on_changed == other.on_changed
    }
}

pub fn Submissions(props: SubmissionsProps) -> Element {
    let SubmissionsProps {
        submissions,
        problem,
        contest_hide_solutions,
        contest_can_manage,
        all_submissions_allowed,
        on_changed,
    } = props;
    let lang = crate::state::language();
    let mut pending = use_signal(|| false);
    let mut details = use_signal(|| None::<Submission>);

    let problem_can_manage = STATE.read().can_manage_problem(&problem);
    let show_download = contest_can_manage || problem_can_manage || !contest_hide_solutions;

    let mut reload = move || {
        if pending() {
            return;
        }
        pending.set(true);
        let all = STATE.read().all_submissions;
        let token = STATE.read().token.clone();
        let pid = problem.id;
        let on_changed = on_changed;
        spawn(async move {
            let res = if all {
                api::problems::get_problem_submissions_all(pid, &token).await
            } else {
                api::problems::get_problem_submissions_my(pid, &token).await
            };
            pending.set(false);
            match res {
                Ok(subs) => {
                    STATE.write().submissions = subs;
                    on_changed.call(());
                }
                Err(e) => crate::alerts::show_alert(crate::alerts::AlertKind::Error, e),
            }
        });
    };

    let details_modal = details().map(|submission| {
        let title = i18n::tr(
            &lang,
            &format!("Посылка #{}", submission.id),
            &format!("Submission #{}", submission.id),
        );
        rsx! {
            div { class: "modal modal-open",
                div { class: "modal-box max-w-4xl", style: "max-height: 85dvh;",
                    div { class: "flex items-center justify-between",
                        h3 { class: "card-title", span { "{title}" } }
                        button { class: "btn btn-sm btn-circle btn-ghost", onclick: move |_| details.set(None), "✕" }
                    }
                    SubmissionDetail { submission: submission.clone(), lang: lang.clone() }
                }
            }
        }
    });

    rsx! {
            div { class: "w-full flex-1 min-h-0 min-w-0",
                div { class: "flex flex-col gap-3 flex-1 min-h-0 min-w-0 h-full",
                    div { class: "flex flex-wrap items-center justify-between gap-4",
                        h3 { class: "card-title", span { "{i18n::tr(&lang, \"Посылки\", \"Submissions\")}" } }
                        div { class: "flex items-center gap-4",
                            if all_submissions_allowed {
                                label { class: "label cursor-pointer justify-start gap-2",
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-sm",
                                        checked: STATE.read().all_submissions,
                                        onchange: move |ev| {
                                            let checked = ev.checked();
                                            STATE.write().all_submissions = checked;
                                            reload();
                                        },
                                    }
                                    span { class: "label-text", "{i18n::tr(&lang, \"все посылки\", \"all submissions\")}" }
                                }
                            }
                            button {
                                class: "btn btn-ghost btn-sm gap-1",
                                disabled: pending(),
                                onclick: move |_| reload(),
                                {icon_element(Icon::Update, 14)}
                                span { "{i18n::tr(&lang, \"обновить\", \"reload\")}" }
                            }
                        }
                    }

                    if submissions.is_empty() {
                        p { class: "italic", "{i18n::tr(&lang, \"Посылок пока что нет\", \"No submissions yet\")}" }
                    } else {
                        div { class: "overflow-x-auto overflow-y-auto flex-1 min-h-0 min-w-0 max-h-[32rem]",
                            table { class: "table table-zebra table-sm",
                                thead {
                                    tr {
                                        th { "#" }
                                        th { "{i18n::tr(&lang, \"когда\", \"when\")}" }
                                        th { "{i18n::tr(&lang, \"кто\", \"who\")}" }
                                        th { "{i18n::tr(&lang, \"язык\", \"language\")}" }
                                        th { "{i18n::tr(&lang, \"вердикт\", \"verdict\")}" }
                                        th { "{i18n::tr(&lang, \"баллы\", \"score\")}" }
                                        th { "{i18n::tr(&lang, \"скачать\", \"download\")}" }
                                        th { "{i18n::tr(&lang, \"подробнее\", \"details\")}" }
                                    }
                                }
                                tbody {
                                    for s in submissions.iter().cloned() {
                                        tr {
                                            td { class: if s.verdict == TestingVerdict::Ok { "font-bold" } else { "" }, "{s.id}" }
                                            td { class: if s.verdict == TestingVerdict::Ok { "font-bold" } else { "" },
                                                DateTimeText { time: s.created_at }
                                            }
                                            td {
                                                UserLink { user_id: s.user_id, username: s.user_login.clone() }
                                            }
                                            td { class: if s.verdict == TestingVerdict::Ok { "font-bold" } else { "" },
                                                {crate::i18n::language_text(&lang, &s.language)}
                                            }
                                            td { VerdictBadge { verdict: s.verdict.clone() } }
                                            td { class: if s.verdict == TestingVerdict::Ok { "font-bold" } else { "" }, "{s.score}" }
                                            td {
                                                if show_download {
                                                    button {
                                                        class: "btn btn-ghost btn-sm gap-1",
                                                        onclick: {
                                                            let token = STATE.read().token.clone();
                                                            let filename = format!("solution.{}", s.language.file_ext());
                                                            let sub = s.clone();
                                                            let on_done = on_changed;
    move |_| {
                                            let token = token.clone();
                                            let filename = filename.clone();
                                            spawn(async move {
                                                if let Err(e) = api::problems::download_submission_file(sub.id, &token, &filename).await {
                                                    crate::alerts::show_alert(crate::alerts::AlertKind::Error, e);
                                                }
                                                on_done.call(());
                                            });
                                        }
                                                        },
                                                        {icon_element(Icon::Download, 14)}
                                                    }
                                                }
                                            }
                                            td {
                                                button {
                                                    class: "btn btn-ghost btn-sm gap-1",
                                                    onclick: move |_| details.set(Some(s.clone())),
                                                    {icon_element(Icon::Info, 14)}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            {details_modal}
        }
}

#[derive(Props, Clone)]
struct SubmissionDetailProps {
    submission: Submission,
    lang: String,
}

impl PartialEq for SubmissionDetailProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.submission, &other.submission)
            && self.lang == other.lang
    }
}

fn SubmissionDetail(props: SubmissionDetailProps) -> Element {
    let SubmissionDetailProps { submission, lang } = props;
    let mut tab = use_signal(|| 0u8); // 0 = subgroups, 1 = tests, 2 = code
    let mut code = use_signal(|| None::<String>);
    // Модалка переиспользуется между посылками — сбрасываем состояние.
    let mut shown_id = use_signal(|| submission.id);
    if shown_id() != submission.id {
        shown_id.set(submission.id);
        tab.set(0);
        code.set(None);
    }
    let code_block = code().map(|src| {
        let ext = submission.language.file_ext();
        let fence = if ext == "!!" {
            String::new()
        } else {
            ext.to_string()
        };
        format!("```{fence}\n{src}\n```")
    });

    rsx! {
        div { class: "w-full mt-3",
            div { class: "tabs tabs-box mb-3 w-fit",
                button {
                    class: if tab() == 0 { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab.set(0),
                    "{i18n::tr(&lang, \"Результаты подгрупп\", \"Subgroup results\")}"
                }
                button {
                    class: if tab() == 1 { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab.set(1),
                    "{i18n::tr(&lang, \"Результаты тестов\", \"Test results\")}"
                }
                button {
                    class: if tab() == 2 { "tab tab-active" } else { "tab" },
                    onclick: move |_| {
                        tab.set(2);
                        if code().is_none() {
                            let token = STATE.read().token.clone();
                            let sid = submission.id;
                            spawn(async move {
                                match api::get_bytes(
                                    &format!("/submissions/{sid}/download"),
                                    &token,
                                )
                                .await
                                {
                                    Ok(bytes) => code.set(Some(
                                        String::from_utf8_lossy(&bytes).into_owned(),
                                    )),
                                    Err(e) => crate::alerts::show_alert(
                                        crate::alerts::AlertKind::Error,
                                        e,
                                    ),
                                }
                            });
                        }
                    },
                    "{i18n::tr(&lang, \"Код\", \"Code\")}"
                }
            }

            div { class: "flex flex-col gap-2 max-h-96 overflow-y-auto",
                if tab() == 0 {
                    for (i, r) in submission.subgroups_results.iter().enumerate() {
                        div { class: "bg-base-300 rounded p-4 text-base-content",
                            p { class: "font-medium", "##{i}" }
                            div { class: "flex justify-between", span { class: "text-sm", "{i18n::tr(&lang, \"вердикт\", \"verdict\")}" }, span { class: "text-sm italic font-medium border border-neutral/50 rounded px-2 py-0.5", "{crate::i18n::subgroup_verdict_text(&lang, &r.verdict)}" } }
                            div { class: "flex justify-between", span { class: "text-sm", "{i18n::tr(&lang, \"тест\", \"test\")}" }, span { class: "text-sm italic", "{r.test}" } }
                            if let Some(score) = r.score {
                                div { class: "flex justify-between", span { class: "text-sm", "{i18n::tr(&lang, \"баллы\", \"score\")}" }, span { class: "text-sm italic", "{score}" } }
                            }
                        }
                    }
                } else if tab() == 1 {
                    for (i, t) in submission.tests_results.iter().enumerate() {
                        div { class: "bg-base-300 rounded p-4 text-base-content",
                            p { class: "font-medium", "##{i + 1}" }
                            div { class: "flex justify-between", span { class: "text-sm", "{i18n::tr(&lang, \"вердикт\", \"verdict\")}" }, span { class: "text-sm italic font-medium border border-neutral/50 rounded px-2 py-0.5", "{crate::i18n::test_verdict_text(&lang, &t.verdict)}" } }
                            if let Some(score) = t.score {
                                div { class: "flex justify-between", span { class: "text-sm", "{i18n::tr(&lang, \"баллы\", \"score\")}" }, span { class: "text-sm italic", "{score}" } }
                            }
                        }
                    }
                } else if let Some(text) = code_block {
                    Markdown { text: text }
                } else {
                    p { class: "italic", "{i18n::tr(&lang, \"загрузка...\", \"loading...\")}" }
                }
            }
        }
    }
}

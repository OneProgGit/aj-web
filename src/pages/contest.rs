use dioxus::prelude::*;
use dioxus_web::WebEventExt;
use wasm_bindgen::JsCast;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        contest_card::ContestCard,
        icon::{icon_element, Icon},
        problem_card::ProblemCard,
        submissions::Submissions,
    },
    i18n,
    models::{problems::PublicProblemConfig, testing::Language},
    pages::{
        contest_announcements::ContestAnnouncements,
        contest_leaderboard::ContestLeaderboard,
        contest_questions::ContestQuestions,
    },
    state::STATE,
};

fn tab_from_url() -> u8 {
    let query = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default();
    if query.contains("tab=posts") {
        1
    } else if query.contains("tab=submissions") {
        4
    } else if query.contains("tab=questions") {
        2
    } else if query.contains("tab=leaderboard") {
        3
    } else {
        crate::state::load_tab()
    }
}

fn tab_to_url(tab: u8) {
    crate::state::save_tab(tab);
    let query = match tab {
        1 => "?tab=posts",
        4 => "?tab=submissions",
        2 => "?tab=questions",
        3 => "?tab=leaderboard",
        _ => "",
    };
    if let Some(window) = web_sys::window() {
        if let Ok(path) = window.location().pathname() {
            let url = format!("{path}{query}");
            if let Ok(history) = window.history() {
                let _ = history
                    .replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
            }
        }
    }
}

/// Shared problem selector (used in main + submissions tabs).
fn problem_selector(problems: Vec<PublicProblemConfig>, mut selected_problem: Signal<usize>) -> Element {
    rsx! {
        select {
            class: "select select-bordered select-sm",
            value: selected_problem().to_string(),
            onchange: move |ev| {
                let idx = ev.value().parse::<usize>().unwrap_or(0);
                selected_problem.set(idx);
                let token = crate::state::token();
                let all = STATE.read().all_submissions;
                spawn(async move {
                    let problems = STATE.read().contest_problems.clone();
                    if let Some(problem) = problems.get(idx) {
                        let res = if all {
                            api::problems::get_problem_submissions_all(problem.id, &token).await
                        } else {
                            api::problems::get_problem_submissions_my(problem.id, &token).await
                        };
                        match res {
                            Ok(subs) => STATE.write().submissions = subs,
                            Err(e) => show_alert(AlertKind::Error, e),
                        }
                    }
                });
            },
            for (i, p) in problems.iter().enumerate() {
                option {
                    value: "{i}",
                    selected: selected_problem() == i,
                    "#{p.index + 1} {i18n::problem_title(&crate::state::language(), &p.name_ru, &p.name_en)}"
                }
            }
        }
    }
}

/// Retest button for the selected problem (admins only, empty otherwise).
fn retest_button(selected: Option<PublicProblemConfig>, mut busy: Signal<bool>) -> Element {
    let lang = crate::state::language();
    rsx! {
        if selected.as_ref().is_some_and(|p| STATE.read().can_manage_problem(p)) {
            button {
                class: "btn btn-ghost btn-sm gap-1",
                disabled: busy(),
                onclick: {
                    let token = STATE.read().token.clone();
                    let pid = selected.as_ref().map(|p| p.id);
                    move |_| {
                        let Some(pid) = pid else { return; };
                        if busy() {
                            return;
                        }
                        busy.set(true);
                        let token = token.clone();
                        spawn(async move {
                            match api::problems::retest_problem(pid, &token).await {
                                Ok(()) => {
                                    busy.set(false);
                                    show_alert(AlertKind::Success, i18n::tr(&crate::state::language(), "Ретест запущен", "Retest started"));
                                }
                                Err(e) => {
                                    busy.set(false);
                                    show_alert(AlertKind::Error, e);
                                }
                            }
                        });
                    }
                },
                {icon_element(Icon::Update, 16)}
                span { "{i18n::tr(&lang, \"ретест\", \"retest\")}" }
            }
        }
    }
}

async fn load_contest_state(
    contest_id: i64,
    problem_index: usize,
    all_submissions: bool,
    force_data: bool,
) {
    let token = crate::state::token();

    if force_data {
        match api::problems::get_contest_problems(contest_id, &token).await {
            Ok(list) => STATE.write().contest_problems = list,
            Err(e) => show_alert(AlertKind::Error, e),
        }
    }

    let problems = STATE.read().contest_problems.clone();
    if let Some(problem) = problems.get(problem_index) {
        let res = if all_submissions {
            api::problems::get_problem_submissions_all(problem.id, &token).await
        } else {
            api::problems::get_problem_submissions_my(problem.id, &token).await
        };
        match res {
            Ok(subs) => STATE.write().submissions = subs,
            Err(e) => show_alert(AlertKind::Error, e),
        }
    }

    if force_data {
        match api::contests::get_contest_leaderboard(contest_id, &token).await {
            Ok(rows) => STATE.write().leaderboard = rows,
            Err(e) => show_alert(AlertKind::Error, e),
        }
        match api::contests::get_contest_posts(contest_id, &token).await {
            Ok(posts) => STATE.write().posts = posts,
            Err(e) => show_alert(AlertKind::Error, e),
        }
        match api::contests::get_contest(contest_id, &token).await {
            Ok(contest) => {
                let mut state = STATE.write();
                if let Some(existing) = state.contests.iter_mut().find(|c| c.id == contest_id) {
                    *existing = contest;
                } else {
                    state.contests.push(contest);
                }
            }
            Err(e) => show_alert(AlertKind::Error, e),
        }
        let can_manage = {
            let state = STATE.read();
            let contest = state.contests.iter().find(|c| c.id == contest_id).cloned();
            contest
                .as_ref()
                .is_some_and(|c| state.can_manage_contest(c))
        };
        let questions = if can_manage {
            api::contests::get_contest_questions_all(contest_id, &token).await
        } else {
            api::contests::get_contest_questions_my(contest_id, &token).await
        };
        match questions {
            Ok(questions) => STATE.write().questions = questions,
            Err(e) => show_alert(AlertKind::Error, e),
        }
    }
}

#[component]
pub fn Contest(contest_id: i64) -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let selected_problem = use_signal(|| 0usize);
    let mut language_idx = use_signal(|| 0usize);
    let mut solution_bytes = use_signal(|| None::<Vec<u8>>);
    let mut solution_name = use_signal(String::new);
    let mut paste_mode = use_signal(|| false);
    let mut paste_code = use_signal(String::new);
    let mut loaded = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut tab = use_signal(tab_from_url);

    if !*loaded.read() {
        STATE.write().all_submissions = false;
        spawn(async move {
            load_contest_state(contest_id, 0, false, true).await;
            crate::components::contest_ws::contest_ws(contest_id);
            loaded.set(true);
        });
        return rsx! {
            div { class: "flex justify-center items-center py-16",
                span { class: "loading loading-spinner loading-lg" }
            }
        };
    }

    let contest = STATE
        .read()
        .contests
        .iter()
        .find(|c| c.id == contest_id)
        .cloned();
    let problems = STATE.read().contest_problems.clone();
    let submissions = STATE.read().submissions.clone();
    let posts_count = STATE.read().posts.len();
    let questions = STATE.read().questions.clone();
    let questions_count = questions.len();
    let questions_answered = questions.iter().filter(|q| !q.answer.is_empty()).count();
    let can_manage = contest
        .as_ref()
        .is_some_and(|c| STATE.read().can_manage_contest(c));

    let selected = problems.get(selected_problem()).cloned();

    let announcements_label = i18n::tr(
        &lang,
        &format!("объявления ({posts_count})"),
        &format!("announcements ({posts_count})"),
    );
    let questions_label = i18n::tr(
        &lang,
        &format!("вопросы ({questions_answered}/{questions_count})"),
        &format!("questions ({questions_answered}/{questions_count})"),
    );
    let pick_label = if !solution_name().is_empty() {        i18n::tr(
            &lang,
            &format!("изменить файл ({})", solution_name()),
            &format!("change file ({})", solution_name()),
        )
    } else {
        i18n::tr(&lang, "выбрать файл", "pick file")
    };

    let paste_mode_label = if paste_mode() {
        i18n::tr(&lang, "файл", "file")
    } else {
        i18n::tr(&lang, "текст", "text")
    };

    rsx! {
        div { class: "flex flex-col gap-4 max-w-7xl mx-auto w-full contest-page", style: "height: calc(100vh - 12rem); height: calc(100dvh - 12rem); min-height: 28rem;",
            div { class: "flex flex-wrap gap-4 items-center",
                button {
                    class: "btn btn-ghost btn-sm gap-2",
                    onclick: move |_| { let _ = navigator.push(crate::Route::Home {}); },
                    {icon_element(Icon::Back, 16)}
                    span { "{i18n::tr(&lang, \"назад\", \"back\")}" }
                }

                div { class: "tabs tabs-lift w-fit",
                    button {
                        class: if tab() == 0 { "tab gap-3 tab-active" } else { "tab gap-3" },
                        onclick: move |_| { tab.set(0); tab_to_url(0); },
                        {icon_element(Icon::Home, 16)}
                        span { "{i18n::tr(&lang, \"главная\", \"main\")}" }
                    }
                    button {
                        class: if tab() == 4 { "tab gap-3 tab-active" } else { "tab gap-3" },
                        onclick: move |_| { tab.set(4); tab_to_url(4); },
                        {icon_element(Icon::PaperPlane, 16)}
                        span { "{i18n::tr(&lang, \"посылки\", \"submissions\")}" }
                    }
                    button {
                        class: if tab() == 1 { "tab gap-3 tab-active" } else { "tab gap-3" },
                        onclick: move |_| { tab.set(1); tab_to_url(1); },
                        {icon_element(Icon::FileText, 16)}
                        span { "{announcements_label}" }
                    }
                    button {
                        class: if tab() == 2 { "tab gap-3 tab-active" } else { "tab gap-3" },
                        onclick: move |_| { tab.set(2); tab_to_url(2); },
                        {icon_element(Icon::QuestionMark, 16)}
                        span { "{questions_label}" }
                    }
                    button {
                        class: if tab() == 3 { "tab gap-3 tab-active" } else { "tab gap-3" },
                        onclick: move |_| { tab.set(3); tab_to_url(3); },
                        {icon_element(Icon::Person, 16)}
                        span { "{i18n::tr(&lang, \"таблица лидеров\", \"leaderboard\")}" }
                    }
                }
            }

            if tab() == 1 {
                ContestAnnouncements { contest_id: contest_id }
            } else if tab() == 2 {
                ContestQuestions { contest_id: contest_id }
            } else if tab() == 3 {
                ContestLeaderboard { contest_id: contest_id }
            } else if tab() == 4 {
                if let Some(contest) = &contest {
                    if let Some(problem) = &selected {
                        div { class: "flex flex-wrap gap-4 items-center",
                            {problem_selector(problems.clone(), selected_problem)}
                            {retest_button(selected.clone(), busy)}
                        }
                        div { class: "flex flex-wrap gap-4 items-center min-h-0",
                        select {
                            class: "select select-bordered select-sm",
                            value: language_idx().to_string(),
                            onchange: move |ev| {
                                language_idx.set(ev.value().parse::<usize>().unwrap_or(0));
                            },
                            option { value: "0", "Rust" }
                            option { value: "1", "C" }
                            option { value: "2", "C++" }
                            option { value: "3", "Go" }
                            option { value: "4", "Python" }
                            option { value: "5", "Pascal" }
                        }

                        button {
                            class: "btn btn-ghost btn-sm gap-1",
                            onclick: move |_| paste_mode.set(!paste_mode()),
                            {icon_element(if paste_mode() { Icon::Upload } else { Icon::Pencil }, 16)}
                            span { "{paste_mode_label}" }
                        }
                        if !paste_mode() {
                            label {
                                r#for: "solution-file",
                                class: "btn btn-ghost btn-sm gap-1",
                                {icon_element(Icon::Upload, 16)}
                                span { "{pick_label}" }
                            }
                        }
                        input {
                            id: "solution-file",
                            r#type: "file",
                            class: "hidden",
                            onchange: move |ev| {
                                let input = ev
                                    .as_web_event()
                                    .target()
                                    .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                let file = input
                                    .as_ref()
                                    .and_then(|el| el.files())
                                    .and_then(|f| f.item(0));
                                if let Some(file) = file {
                                    let name = file.name();
                                    spawn(async move {
                                        let buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await;
                                        match buffer {
                                            Ok(b) => {
                                                let view = js_sys::Uint8Array::new(&b);
                                                solution_bytes.set(Some(view.to_vec()));
                                                solution_name.set(name);
                                            }
                                            Err(_) => show_alert(AlertKind::Error, crate::i18n::tr(&crate::state::language(), "Не удалось прочитать файл", "Could not read file")),
                                        }
                                    });
                                }
                                if let Some(input) = input {
                                    input.set_value("");
                                }
                            },
                        }

                        if paste_mode() {
                            textarea {
                                class: "textarea textarea-bordered w-full font-mono",
                                rows: "8",
                                placeholder: i18n::tr(&lang, "вставьте код решения", "paste solution code"),
                                value: paste_code(),
                                oninput: move |ev| paste_code.set(ev.value()),
                            }
                        }

                        button {
                            class: "btn btn-primary btn-sm gap-1",
                            disabled: busy()
                                || if paste_mode() {
                                    paste_code().trim().is_empty()
                                } else {
                                    solution_bytes().is_none()
                                },
                            onclick: {
                                let token = STATE.read().token.clone();
                                let contest_id = contest.id;
                                let problem_id = selected.as_ref().map(|p| p.id);
                                move |_| {
                                    let Some(problem_id) = problem_id else { return; };
                                    let bytes = if paste_mode() {
                                        let code = paste_code();
                                        if code.trim().is_empty() {
                                            return;
                                        }
                                        code.into_bytes()
                                    } else {
                                        let Some(bytes) = solution_bytes().clone() else { return };
                                        bytes
                                    };
                                    if busy() {
                                        return;
                                    }
                                    busy.set(true);
                                    let language = match language_idx() {
                                        0 => Language::Rust,
                                        1 => Language::C,
                                        2 => Language::Cpp,
                                        3 => Language::Go,
                                        4 => Language::Python,
                                        _ => Language::FreePascal,
                                    };
                                    let idx = selected_problem();
                                    let token = token.clone();
                                    spawn(async move {
                                        match api::problems::submit_solution(contest_id, problem_id, language, bytes, &token).await {
                                            Ok(id) => {
                                                busy.set(false);
                                                show_alert(AlertKind::Success, i18n::tr(&crate::state::language(), &format!("Посылка #{id} отправлена"), &format!("Submission #{id} sent")));
                                                let all = STATE.read().all_submissions;
                                                load_contest_state(contest_id, idx, all, false).await;
                                            }
                                            Err(e) => {
                                                busy.set(false);
                                                show_alert(AlertKind::Error, e);
                                            }
                                        }
                                    });
                                }
                            },
                            {icon_element(Icon::PaperPlane, 16)}
                            span { "{i18n::tr(&lang, \"отослать\", \"send\")}" }
                        }
                        }
                            div { class: "flex flex-col gap-4 flex-1 min-h-0 min-w-0",
                                Submissions {
                                    submissions: submissions.clone(),
                                    problem: problem.clone(),
                                    contest_hide_solutions: contest.solutions_hidden,
                                    contest_can_manage: can_manage,
                                    all_submissions_allowed: can_manage,
                                    on_changed: move |_| {},
                                }
                            }
                        } else {
                        p { class: "italic", "{i18n::tr(&lang, \"Нет задач\", \"No problems\")}" }
                    }
                }
            } else if let Some(contest) = &contest {
                div { class: "grid lg:grid-cols-2 gap-4 w-full items-stretch flex-1 min-h-0",
                    div { class: "flex flex-col gap-4 min-h-0 min-w-0",
                        ContestCard {
                            contest: contest.clone(),
                            show_enter: false,
                            compact: false,
                            on_changed: move |_| {
                                let token = crate::state::token();
                                spawn(async move {
                                    match api::contests::get_contest(contest_id, &token).await {
                                        Ok(fresh) => {
                                            let mut state = STATE.write();
                                            if let Some(slot) =
                                                state.contests.iter_mut().find(|c| c.id == contest_id)
                                            {
                                                *slot = fresh;
                                            } else {
                                                state.contests.push(fresh);
                                            }
                                        }
                                        Err(e) => show_alert(AlertKind::Error, e),
                                    }
                                });
                            },
                        }
                        div { class: "flex flex-wrap gap-4 items-center",
                            {problem_selector(problems.clone(), selected_problem)}
                            {retest_button(selected.clone(), busy)}
                        }
                    }
                    if let Some(problem) = &selected {
                        div { class: "flex flex-col gap-4 min-h-0 min-w-0",
                            ProblemCard {
                                problem: problem.clone(),
                                show_id: false,
                                subgroups_scroll: true,
                                on_changed: move |_| {},
                            }
                        }
                    }
                }
            } else {
                p { class: "italic", "{i18n::tr(&lang, \"Контест не найден\", \"Contest not found\")}" }
            }
        }
    }
}

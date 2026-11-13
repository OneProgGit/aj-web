use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    api::problems::ProblemArchiveError,
    components::markdown::Markdown,
    i18n,
};

/// Compiler log of the last failed checker build.
static CHECKER_LOG: GlobalSignal<Option<String>> = GlobalSignal::new(|| None);

fn title(lang: &str) -> String {
    i18n::tr(
        lang,
        "Ошибка компиляции чекера",
        "Checker compilation error",
    )
}

fn close() {
    *CHECKER_LOG.write() = None;
}

/// Wraps the log into a fenced code block, choosing a fence longer than any
/// backtick run inside the log so compiler output cannot terminate it early.
fn code_block(log: &str) -> String {
    let mut longest_run = 0;
    let mut current_run = 0;
    for ch in log.chars() {
        if ch == '`' {
            current_run += 1;
            longest_run = longest_run.max(current_run);
        } else {
            current_run = 0;
        }
    }
    let fence = "`".repeat((longest_run + 1).max(3));
    format!("{fence}text\n{log}\n{fence}")
}

/// Shows a checker build failure. An empty compiler log is not worth a modal:
/// a plain toast saying that compilation failed is enough.
pub fn show_checker_compilation_error(log: String) {
    let log = log.trim_end().to_owned();
    if log.trim().is_empty() {
        show_alert(AlertKind::Error, title(&crate::state::language()));
        return;
    }
    *CHECKER_LOG.write() = Some(log);
}

/// Routes a problem archive operation error to the appropriate UI.
pub fn show_problem_archive_error(error: ProblemArchiveError) {
    match error {
        ProblemArchiveError::Message(message) => show_alert(AlertKind::Error, message),
        ProblemArchiveError::CheckerCompilation(log) => show_checker_compilation_error(log),
    }
}

/// Modal with the checker compiler log as a copyable code block.
#[component]
pub fn CompilerErrorHost() -> Element {
    let lang = crate::state::language();
    let Some(log) = CHECKER_LOG.read().clone() else {
        return rsx! {};
    };

    rsx! {
        div { class: "modal modal-open",
            div { class: "modal-box max-w-4xl flex flex-col gap-3",
                div { class: "flex items-start justify-between gap-3",
                    div {
                        h3 { class: "text-lg font-bold", "{title(&lang)}" }
                        p { class: "text-sm opacity-70 mt-1",
                            "{i18n::tr(&lang, \"Лог компилятора можно скопировать целиком.\", \"You can copy the whole compiler log.\")}"
                        }
                    }
                    button {
                        class: "btn btn-sm btn-circle btn-ghost",
                        r#type: "button",
                        onclick: move |_| close(),
                        "✕"
                    }
                }
                div { class: "md-code-scroll",
                    Markdown { text: code_block(&log) }
                }
                div { class: "modal-action",
                    button {
                        class: "btn btn-primary btn-sm",
                        r#type: "button",
                        onclick: move |_| close(),
                        "{i18n::tr(&lang, \"Закрыть\", \"Close\")}"
                    }
                }
            }
            div { class: "modal-backdrop", onclick: move |_| close() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::code_block;

    #[test]
    fn fence_is_longer_than_backticks_in_log() {
        let log = "error: expected ```\n  at src/main.rs";
        let block = code_block(log);
        assert!(block.starts_with("````text\n"));
        assert!(block.ends_with("\n````"));
        assert!(block.contains(log));
    }
}

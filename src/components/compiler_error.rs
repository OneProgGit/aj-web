use dioxus::prelude::*;

use crate::{
    alerts::{AlertKind, show_alert},
    api::problems::ProblemArchiveError,
    components::markdown::{Markdown, plain_text_code_block},
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
                    h3 { class: "text-lg font-bold", "{title(&lang)}" }
                    button {
                        class: "btn btn-sm btn-circle btn-ghost",
                        r#type: "button",
                        onclick: move |_| close(),
                        "✕"
                    }
                }
                div { class: "md-code-scroll",
                    Markdown { text: plain_text_code_block(&log) }
                }
            }
            div { class: "modal-backdrop", onclick: move |_| close() }
        }
    }
}

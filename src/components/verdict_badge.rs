use dioxus::prelude::*;

use crate::models::verdicts::TestingVerdict;

/// A colored daisyUI badge for a submission verdict.
/// `Processing` / `Pending` are shown with an animation, matching aj-app.
#[component]
pub fn VerdictBadge(verdict: TestingVerdict) -> Element {
    let lang = crate::state::language();
    let (classes, text, animated) = match verdict {
        TestingVerdict::Pending => (
            "badge badge-warning px-4 whitespace-nowrap",
            crate::i18n::tr_ru(&lang, "В очереди"),
            true,
        ),
        TestingVerdict::Compiling => (
            "badge badge-info px-4 whitespace-nowrap",
            crate::i18n::tr_ru(&lang, "Компиляция"),
            true,
        ),
        TestingVerdict::Testing => (
            "badge badge-info px-4 whitespace-nowrap",
            crate::i18n::tr_ru(&lang, "Тестирование"),
            true,
        ),
        TestingVerdict::Ok => (
            "badge badge-success px-4 rounded-lg whitespace-nowrap",
            verdict.to_string(),
            false,
        ),
        TestingVerdict::PartialSolution => (
            "badge badge-warning px-4 whitespace-nowrap",
            crate::i18n::tr_ru(&lang, "Частичное решение"),
            false,
        ),
        TestingVerdict::CompilationError => (
            "badge badge-error px-4 whitespace-nowrap",
            crate::i18n::tr_ru(&lang, "Ошибка компиляции"),
            false,
        ),
        TestingVerdict::Fail => (
            "badge badge-error px-4 whitespace-nowrap",
            crate::i18n::tr(&lang, "Баг", "Bug"),
            false,
        ),
    };
    rsx! {
        span {
            class: "{classes}",
            class: if animated { "badge-ghost" } else { "" },
            "{text}"
        }
    }
}

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
            crate::i18n::tr(&lang, "В очереди", "Pending"),
            true,
        ),
        TestingVerdict::Compiling => (
            "badge badge-info px-4 whitespace-nowrap",
            crate::i18n::tr(&lang, "Компиляция", "Compiling"),
            true,
        ),
        TestingVerdict::Testing => (
            "badge badge-info px-4 whitespace-nowrap",
            crate::i18n::tr(&lang, "Тестирование", "Testing"),
            true,
        ),
        TestingVerdict::Ok => (
            "badge badge-success px-4 rounded-lg whitespace-nowrap",
            crate::i18n::tr(&lang, "Полное решение", "Accepted"),
            false,
        ),
        TestingVerdict::PartialSolution => (
            "badge badge-warning px-4 whitespace-nowrap",
            crate::i18n::tr(&lang, "Частичное решение", "Partial solution"),
            false,
        ),
        TestingVerdict::CompilationError => (
            "badge badge-error px-4 whitespace-nowrap",
            crate::i18n::tr(&lang, "Ошибка компиляции", "Compilation error"),
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

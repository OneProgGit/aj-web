use dioxus::prelude::*;

use crate::models::verdicts::TestingVerdict;

#[must_use]
pub fn verdict_label(lang: &str, verdict: &TestingVerdict) -> String {
    match verdict {
        TestingVerdict::Pending => crate::i18n::tr(lang, "В очереди", "Pending"),
        TestingVerdict::Compiling => crate::i18n::tr(lang, "Компиляция", "Compiling"),
        TestingVerdict::Testing => crate::i18n::tr(lang, "Тестирование", "Testing"),
        TestingVerdict::Ok => crate::i18n::tr(lang, "Полное решение", "Accepted"),
        TestingVerdict::PartialSolution => {
            crate::i18n::tr(lang, "Частичное решение", "Partial solution")
        }
        TestingVerdict::CompilationError => {
            crate::i18n::tr(lang, "Ошибка компиляции", "Compilation error")
        }
        TestingVerdict::Fail => crate::i18n::tr(lang, "Баг", "Bug"),
    }
}

/// A colored daisyUI badge for a submission verdict.
/// `Processing` / `Pending` are shown with an animation, matching aj-app.
#[component]
pub fn VerdictBadge(verdict: TestingVerdict) -> Element {
    let lang = crate::state::language();
    let text = verdict_label(&lang, &verdict);
    let (classes, animated) = match verdict {
        TestingVerdict::Pending => ("badge badge-warning px-4 whitespace-nowrap", true),
        TestingVerdict::Compiling => ("badge badge-info px-4 whitespace-nowrap", true),
        TestingVerdict::Testing => ("badge badge-info px-4 whitespace-nowrap", true),
        TestingVerdict::Ok => (
            "badge badge-success px-4 rounded-lg whitespace-nowrap",
            false,
        ),
        TestingVerdict::PartialSolution => ("badge badge-warning px-4 whitespace-nowrap", false),
        TestingVerdict::CompilationError => ("badge badge-error px-4 whitespace-nowrap", false),
        TestingVerdict::Fail => ("badge badge-error px-4 whitespace-nowrap", false),
    };
    rsx! {
        span {
            class: "{classes}",
            class: if animated { "badge-ghost" } else { "" },
            "{text}"
        }
    }
}

/// Minimal tr() dictionary. The original app bundles en/ru Slint translations;
/// this rewrite keeps the same Russian (default) texts and adds English where used.
pub fn tr(lang: &str, ru: &str, en: &str) -> String {
    if lang == "en" {
        en.to_string()
    } else {
        ru.to_string()
    }
}

pub fn tr_ru(_lang: &str, ru: &str) -> String {
    ru.to_string()
}

pub fn problem_title(lang: &str, name_ru: &str, name_en: &str) -> String {
    if lang == "en" && !name_en.is_empty() {
        name_en.to_string()
    } else {
        name_ru.to_string()
    }
}

#[allow(dead_code)]
pub const fn status_text_before_start() -> &'static str {
    "до начала"
}
#[allow(dead_code)]
pub const fn status_text_ongoing() -> &'static str {
    "идёт сейчас"
}
#[allow(dead_code)]
pub const fn status_text_finished() -> &'static str {
    "завершён"
}
#[allow(dead_code)]
pub const fn status_text_upsolving() -> &'static str {
    "дорешка"
}

#[allow(dead_code)]
pub fn admin_level_text(lang: &str, level: &crate::models::users::AdminLevel) -> String {
    match level {
        crate::models::users::AdminLevel::User => tr(lang, "пользователь", "user"),
        crate::models::users::AdminLevel::Admin => tr(lang, "админ", "admin"),
        crate::models::users::AdminLevel::Owner => tr(lang, "владелец", "owner"),
    }
}

pub fn language_text(lang: &str, language: &crate::models::testing::Language) -> String {
    match language {
        crate::models::testing::Language::C => "C".into(),
        crate::models::testing::Language::Cpp => "C++".into(),
        crate::models::testing::Language::Go => "Go".into(),
        crate::models::testing::Language::Rust => "Rust".into(),
        crate::models::testing::Language::Python => "Python".into(),
        crate::models::testing::Language::FreePascal => "Pascal".into(),
        crate::models::testing::Language::Unknown => {
            if lang == "en" {
                "Unknown".into()
            } else {
                "Неизвестно".into()
            }
        }
    }
}

#[allow(dead_code)]
pub fn problem_type_text(lang: &str, r#type: &crate::models::problems::ProblemType) -> String {
    let ru = match r#type {
        crate::models::problems::ProblemType::Default => "Обычная",
        crate::models::problems::ProblemType::Interactive => "Интерактивная",
        crate::models::problems::ProblemType::RunTwice => "Запусти дважды",
        crate::models::problems::ProblemType::InteractiveRunTwice => "Интерактивная, запуск дважды",
        crate::models::problems::ProblemType::RunTwiceFirstInteractive => {
            "Дважды, интерактивный первый"
        }
        crate::models::problems::ProblemType::RunTwiceSecondInteractive => {
            "Дважды, интерактивный второй"
        }
    };
    let en = match r#type {
        crate::models::problems::ProblemType::Default => "Default",
        crate::models::problems::ProblemType::Interactive => "Interactive",
        crate::models::problems::ProblemType::RunTwice => "Run twice",
        crate::models::problems::ProblemType::InteractiveRunTwice => "Interactive, run twice",
        crate::models::problems::ProblemType::RunTwiceFirstInteractive => {
            "Run twice, first interactive"
        }
        crate::models::problems::ProblemType::RunTwiceSecondInteractive => {
            "Run twice, second interactive"
        }
    };
    tr(lang, ru, en)
}

#[allow(dead_code)]
pub fn testing_type_text(lang: &str, t: &crate::models::problems::ProblemTestingType) -> String {
    match t {
        crate::models::problems::ProblemTestingType::Ioi => {
            tr(lang, "IOI (группы)", "IOI (subgroups)")
        }
        crate::models::problems::ProblemTestingType::IoiMergeSubgroups => {
            tr(lang, "IOI (объединять группы)", "IOI (merge subgroups)")
        }
    }
}

/// Human verdict text for a submission (mirrors aj-submissions.slint).
#[allow(dead_code)]
pub fn total_verdict_text(lang: &str, v: &crate::models::verdicts::TestingVerdict) -> String {
    match v {
        crate::models::verdicts::TestingVerdict::Ok => tr(lang, "полное решение", "full solution"),
        crate::models::verdicts::TestingVerdict::PartialSolution => {
            tr(lang, "частичное решение", "partial solution")
        }
        crate::models::verdicts::TestingVerdict::Pending => tr(lang, "в очереди", "pending"),
        crate::models::verdicts::TestingVerdict::Compiling => {
            tr(lang, "компилируется", "compiling")
        }
        crate::models::verdicts::TestingVerdict::CompilationError => {
            tr(lang, "ошибка компиляции", "compilation error")
        }
        crate::models::verdicts::TestingVerdict::Testing => tr(lang, "тестируется", "testing"),
        crate::models::verdicts::TestingVerdict::Fail => tr(lang, "баг", "bug"),
    }
}

/// Human verdict text for a subgroup result (mirrors aj-subgroup-result-card).
/// Russian agrees in gender with «подгруппа» (feminine).
pub fn subgroup_verdict_text(lang: &str, v: &crate::models::verdicts::Verdict) -> String {
    match v {
        crate::models::verdicts::Verdict::Ok => tr(lang, "пройдена", "passed"),
        crate::models::verdicts::Verdict::RuntimeError => {
            tr(lang, "ошибка выполнения", "runtime error")
        }
        crate::models::verdicts::Verdict::TimeLimitExceeded => tr(
            lang,
            "превышено ограничение по времени",
            "time limit exceeded",
        ),
        crate::models::verdicts::Verdict::MemoryLimitExceeded => tr(
            lang,
            "превышено ограничение по памяти",
            "memory limit exceeded",
        ),
        crate::models::verdicts::Verdict::SecurityError => {
            tr(lang, "ошибка безопасности", "security error")
        }
        crate::models::verdicts::Verdict::WrongAnswer => {
            tr(lang, "неправильный ответ", "wrong answer")
        }
        crate::models::verdicts::Verdict::PresentationError => {
            tr(lang, "неправильный формат вывода", "presentation error")
        }
        crate::models::verdicts::Verdict::Skipped => tr(lang, "пропущена", "skipped"),
        crate::models::verdicts::Verdict::Testing => tr(lang, "тестируется", "testing"),
        crate::models::verdicts::Verdict::Fail => tr(lang, "баг", "bug"),
    }
}

/// Human verdict text for an individual test result (mirrors
/// aj-test-result-card). Russian agrees in gender with «тест» (masculine).
pub fn test_verdict_text(lang: &str, v: &crate::models::verdicts::Verdict) -> String {
    match v {
        crate::models::verdicts::Verdict::Ok => tr(lang, "пройден", "passed"),
        crate::models::verdicts::Verdict::Skipped => tr(lang, "пропущен", "skipped"),
        _ => subgroup_verdict_text(lang, v),
    }
}

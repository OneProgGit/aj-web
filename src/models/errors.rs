pub use aj_models::errors::{
    AdaJudgeError, AuthError, Contest, Deletion, InvalidProblem,
};

pub fn describe_error(error: &AdaJudgeError) -> String {
    let lang = crate::state::language();
    let tr = |ru: &str, en: &str| crate::i18n::tr(&lang, ru, en);
    match error {
        AdaJudgeError::NotFound => tr("Не найдено", "Not found"),
        AdaJudgeError::Internal => tr("Внутренняя ошибка", "Internal error"),
        AdaJudgeError::InvalidProblem(problem) => match problem {
            InvalidProblem::SubgroupConflict { subgroup, depends_on } => {
                if lang == "en" {
                    format!("Subgroup {subgroup} depends on subgroup {depends_on}")
                } else {
                    format!("Группа {subgroup} зависит от группы {depends_on}")
                }
            }
            InvalidProblem::InvalidSubgroupScoring { subgroup } => {
                if lang == "en" {
                    format!("Subgroup {subgroup}: both score and score_per_test are set (or neither)")
                } else {
                    format!("Группа {subgroup}: заданы и score, и score_per_test (или ни одно)")
                }
            }
            InvalidProblem::MissingConfig => tr("Конфигурация задачи не найдена", "Problem config not found"),
            InvalidProblem::TomlError { message } => {
                if lang == "en" {
                    format!("TOML error: {message}")
                } else {
                    format!("Ошибка TOML: {message}")
                }
            }
            InvalidProblem::OwnerId => tr("Неверный или отсутствующий владелец", "Invalid or missing owner"),
            InvalidProblem::CheckerCompilationError => {
                tr("Ошибка компиляции чекера", "Checker compilation error")
            }
        },
        AdaJudgeError::InvalidJwt => tr("Неверный токен", "Invalid token"),
        AdaJudgeError::Auth(auth) => match auth {
            AuthError::InvalidLoginOrPassword => tr("Неверный логин или пароль", "Invalid login or password"),
            AuthError::AlreadyExists => tr("Пользователь уже существует", "User already exists"),
            AuthError::PasswordsDontMatch => tr("Пароли не совпадают", "Passwords do not match"),
        },
        AdaJudgeError::Deletion(deletion) => match deletion {
            Deletion::InvalidLoginOrPassword => tr("Неверный логин или пароль", "Invalid login or password"),
            Deletion::MissingDeletionConfirmation => {
                tr("Нет подтверждения удаления", "Missing deletion confirmation")
            }
        },
        AdaJudgeError::Forbidden => tr("Доступ запрещён", "Forbidden"),
        AdaJudgeError::Contest(contest) => match contest {
            Contest::Time => tr("Время начала >= времени конца", "Start time is >= finish time"),
        },
        AdaJudgeError::BadRequest => tr("Неверный запрос", "Bad request"),
    }
}

use dioxus::prelude::*;

use crate::models::users::AdminLevel;

/// Badge showing the user's admin level. Mirrors `AdminBadge` in aj-app.
/// Цвета — контейнерные роли Material 3 ( tonal / error / primary ),
/// поэтому бейджи адаптируются к светлой и тёмной темам.
#[component]
pub fn AdminBadge(level: AdminLevel, lang: String) -> Element {
    let (bg, text, fg) = match level {
        AdminLevel::User => (
            "var(--md-surface-container-highest)",
            crate::i18n::tr(&lang, "пользователь", "user"),
            "var(--color-base-content)",
        ),
        AdminLevel::Admin => (
            "var(--color-error)",
            crate::i18n::tr(&lang, "админ", "admin"),
            "var(--color-error-content)",
        ),
        AdminLevel::Owner => (
            "var(--color-primary)",
            crate::i18n::tr(&lang, "владелец", "owner"),
            "var(--color-primary-content)",
        ),
    };
    rsx! {
        span {
            class: "badge font-medium",
            style: "background-color: {bg}; color: {fg}; border-color: transparent;",
            "{text}"
        }
    }
}

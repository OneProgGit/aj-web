use dioxus::prelude::*;

use crate::models::users::AdminLevel;

/// Badge showing the user's admin level. Mirrors `AdminBadge` in aj-app.
#[component]
pub fn AdminBadge(level: AdminLevel, lang: String) -> Element {
    let (bg, text) = match level {
        AdminLevel::User => ("#b7b0e7", crate::i18n::tr(&lang, "пользователь", "user")),
        AdminLevel::Admin => ("#ff7302", crate::i18n::tr(&lang, "админ", "admin")),
        AdminLevel::Owner => ("#ffbf11", crate::i18n::tr(&lang, "владелец", "owner")),
    };
    rsx! {
        span {
            class: "badge font-bold",
            style: "background-color: {bg}; color: #273a72; border-color: transparent;",
            "{text}"
        }
    }
}

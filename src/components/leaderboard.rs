use dioxus::prelude::*;

use crate::{
    i18n,
    models::{contests::LeaderboardRow, problems::PublicProblemConfig},
};

use super::user_link::UserLink;

/// Leaderboard table. Mirrors `AJLeaderboard`.
#[derive(Props, Clone)]
pub struct LeaderboardProps {
    pub rows: Vec<LeaderboardRow>,
    pub problems: Vec<PublicProblemConfig>,
    pub contest_id: i64,
    pub can_manage: bool,
}

impl PartialEq for LeaderboardProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.rows, &other.rows)
            && crate::models::props_json_eq(&self.problems, &other.problems)
            && self.contest_id == other.contest_id
            && self.can_manage == other.can_manage
    }
}

pub fn Leaderboard(props: LeaderboardProps) -> Element {
    let LeaderboardProps {
        rows,
        problems,
        can_manage,
        ..
    } = props;
    let lang = crate::state::language();

    rsx! {
        div { class: "flex flex-col gap-3 w-full max-w-3xl",
                if rows.is_empty() {
                    p { class: "italic",
                        {if can_manage {
                            i18n::tr(&lang, "В таблице лидеров пока ничего нет", "Leaderboard is empty")
                        } else {
                            i18n::tr(&lang, "Таблица лидеров недоступна до конца контеста", "Leaderboard unavailable until the contest finishes")
                        }}
                    }
                } else {
                    div { class: "overflow-x-auto overflow-y-auto max-h-[32rem] min-w-0",
                        table { class: "table table-zebra table-sm",
                            thead {
                                tr {
                                    th { "#" }
                                    th { "{i18n::tr(&lang, \"кто\", \"who\")}" }
                                    for p in &problems { th { "{p.index + 1}" } }
                                    th { "=" }
                                }
                            }
                            tbody {
                                for (place, row) in rows.iter().enumerate() {
                                    tr {
                                        td { "{place + 1}" }
                                        td {
                                            UserLink { user_id: row.user_id, username: row.user_login.clone() }
                                        }
                                        for score in &row.scores {
                                            td {
                                                class: if score.is_some_and(|s| s == 100.0) { "font-bold" } else { "" },
                                                {score.map_or_else(String::new, |s| s.to_string())}
                                            }
                                        }
                                        td { class: "font-bold", "{row.total_score}" }
                                    }
                                }
                            }
                        }
                    }
                }
        }
    }
}

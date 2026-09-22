use dioxus::prelude::*;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        icon::{icon_element, Icon},
        leaderboard::Leaderboard,
    },
    i18n,
    state::STATE,
};

#[component]
pub fn ContestLeaderboard(contest_id: i64) -> Element {
    let lang = crate::state::language();
    let mut loaded = use_signal(|| false);

    if !*loaded.read() {
        loaded.set(true);
        spawn(async move {
            let token = crate::state::token();
            match api::contests::get_contest(contest_id, &token).await {
                Ok(contest) => {
                    let mut state = STATE.write();
                    if let Some(existing) = state.contests.iter_mut().find(|c| c.id == contest_id) {
                        *existing = contest;
                    } else {
                        state.contests.push(contest);
                    }
                }
                Err(e) => show_alert(AlertKind::Error, e),
            }
            match api::problems::get_contest_problems(contest_id, &token).await {
                Ok(list) => STATE.write().contest_problems = list,
                Err(e) => show_alert(AlertKind::Error, e),
            }
            match api::contests::get_contest_leaderboard(contest_id, &token).await {
                Ok(rows) => STATE.write().leaderboard = rows,
                Err(e) => show_alert(AlertKind::Error, e),
            }
        });
    }

    let contest = STATE
        .read()
        .contests
        .iter()
        .find(|c| c.id == contest_id)
        .cloned();
    let can_manage = contest
        .as_ref()
        .is_some_and(|c| STATE.read().can_manage_contest(c));
    let problems = STATE.read().contest_problems.clone();
    let leaderboard = STATE.read().leaderboard.clone();

    let export_lang = lang.clone();
    let export = std::rc::Rc::new(move |fmt: &'static str| {
        let esc = |s: &str| {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        };
        let mut header = vec![
            "#".to_string(),
            "id".to_string(),
            i18n::tr(&export_lang, "кто", "who"),
        ];
        if !leaderboard.is_empty() {
            for i in 1..=problems.len() {
                header.push(i.to_string());
            }
        }
        header.push("=".to_string());
        let mut table: Vec<Vec<String>> = Vec::new();
        for (place, row) in leaderboard.iter().enumerate() {
            let mut fields = vec![
                (place + 1).to_string(),
                row.user_id.to_string(),
                row.user_login.clone(),
            ];
            fields.extend(
                row.scores
                    .iter()
                    .map(|s| s.map_or_else(String::new, |v| v.to_string())),
            );
            fields.push(row.total_score.to_string());
            table.push(fields);
        }
        if fmt == "md" {
            let esc_md = |s: &str| s.replace('|', "\\|");
            let mut md = String::new();
            md.push_str(&format!(
                "| {} |\n|{}|\n",
                header.iter().map(|h| esc_md(h)).collect::<Vec<_>>().join(" | "),
                header.iter().map(|_| "---").collect::<Vec<_>>().join("|"),
            ));
            for row in &table {
                md.push_str(&format!(
                    "| {} |\n",
                    row.iter().map(|c| esc_md(c)).collect::<Vec<_>>().join(" | "),
                ));
            }
            crate::api::trigger_download(md.into_bytes(), "leaderboard.md");
        } else if fmt == "html" {
            let mut doc = String::from(
                "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Leaderboard</title><style>table{border-collapse:collapse}th,td{border:1px solid #999;padding:4px 8px;text-align:center}</style></head><body><table><thead><tr>",
            );
            for h in &header {
                doc.push_str(&format!("<th>{}</th>", esc(h)));
            }
            doc.push_str("</tr></thead><tbody>");
            for row in &table {
                doc.push_str("<tr>");
                for cell in row {
                    doc.push_str(&format!("<td>{}</td>", esc(cell)));
                }
                doc.push_str("</tr>");
            }
            doc.push_str("</tbody></table></body></html>");
            crate::api::trigger_download(doc.into_bytes(), "leaderboard.html");
        } else {
            let mut csv = String::new();
            csv.push_str(&header.join(","));
            csv.push('\n');
            for row in &table {
                csv.push_str(&row.join(","));
                csv.push('\n');
            }
            crate::api::trigger_download(csv.into_bytes(), "leaderboard.csv");
        }
    });

    let rows = STATE.read().leaderboard.clone();
    let table_problems = STATE.read().contest_problems.clone();

    rsx! {
        div { class: "flex flex-col gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex flex-wrap gap-4 items-center",
                h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Таблица лидеров\", \"Leaderboard\")}" }
                if can_manage {
                    details { class: "dropdown",
                        summary { class: "btn btn-ghost btn-sm gap-1",
                            {icon_element(Icon::Download, 14)}
                            span { "{i18n::tr(&lang, \"экспортировать\", \"export\")}" }
                        }
                        ul { class: "menu dropdown-content bg-base-200 rounded-box p-2 shadow",
                            li {
                                button {
                                    onclick: {
                                        let export = export.clone();
                                        move |_| export("csv")
                                    },
                                    "CSV"
                                }
                            }
                            li {
                                button {
                                    onclick: {
                                        let export = export.clone();
                                        move |_| export("md")
                                    },
                                    "MD"
                                }
                            }
                            li {
                                button {
                                    onclick: {
                                        let export = export.clone();
                                        move |_| export("html")
                                    },
                                    "HTML"
                                }
                            }
                        }
                    }
                }
            }

            Leaderboard {
                rows: rows,
                problems: table_problems,
                contest_id: contest_id,
                can_manage: can_manage,
            }
        }
    }
}

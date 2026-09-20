use crate::models::{
    contests::{ContestPost, LeaderboardRow, PublicContestConfig},
    problems::{ProblemQuestion, PublicProblemConfig},
    testing::Submission,
    users::PrivateUserData,
};
use chrono::Utc;
use dioxus::prelude::*;
use std::cmp::Ordering;

/// Mirrors `AJContestStatus` in aj-app.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContestStatus {
    BeforeStart,
    Ongoing,
    Finished,
    Upsolving,
}

pub fn contest_status(contest: &PublicContestConfig) -> ContestStatus {
    let now = Utc::now();
    match now.cmp(&contest.starts_at) {
        Ordering::Less => ContestStatus::BeforeStart,
        _ => match now.cmp(&contest.finishes_at) {
            Ordering::Less | Ordering::Equal => ContestStatus::Ongoing,
            _ => {
                if contest.upsolving_enabled {
                    ContestStatus::Upsolving
                } else {
                    ContestStatus::Finished
                }
            }
        },
    }
}

impl ContestStatus {
    #[must_use]
    pub const fn css_bg(&self) -> &'static str {
        match self {
            Self::BeforeStart => "bg-accent",
            Self::Ongoing => "bg-success",
            Self::Finished => "bg-primary",
            Self::Upsolving => "bg-secondary",
        }
    }

    pub const fn css_text(&self) -> &'static str {
        match self {
            Self::BeforeStart => "text-accent-content",
            Self::Ongoing => "text-success-content",
            Self::Finished => "text-primary-content",
            Self::Upsolving => "text-secondary-content",
        }
    }
}

const TOKEN_KEY: &str = "aj_web_token";
const LANG_KEY: &str = "aj_web_lang";

#[derive(Clone, Debug, Default)]
pub struct GlobalState {
    pub token: Option<String>,
    pub user: Option<PrivateUserData>,
    pub language: String,
    pub contests: Vec<PublicContestConfig>,
#[allow(dead_code)]
    pub my_contests: Vec<PublicContestConfig>,
    pub contests_is_all: bool,
    pub contest_problems: Vec<PublicProblemConfig>,
    pub posts: Vec<ContestPost>,
    pub questions: Vec<ProblemQuestion>,
    pub submissions: Vec<Submission>,
    pub all_submissions: bool,
    pub leaderboard: Vec<LeaderboardRow>,
    pub problems: Vec<PublicProblemConfig>,
    pub users: Vec<PrivateUserData>,
}

impl GlobalState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            token: load_token(),
            language: load_language(),
            contests_is_all: true,
            ..Self::default()
        }
    }

    pub fn is_admin(&self) -> bool {
        self.user
            .as_ref()
            .is_some_and(|u| u.admin_level != crate::models::users::AdminLevel::User)
    }

    pub fn is_owner(&self) -> bool {
        self.user
            .as_ref()
            .is_some_and(|u| u.admin_level == crate::models::users::AdminLevel::Owner)
    }

    pub fn can_manage_contest(&self, contest: &PublicContestConfig) -> bool {
        let Some(user) = &self.user else {
            return false;
        };
        contest.owner_id.is_some_and(|owner_id| owner_id == user.id)
            || user.admin_level == crate::models::users::AdminLevel::Owner
    }

    pub fn can_enter_contest(&self, contest: &PublicContestConfig) -> bool {
        let Some(user) = &self.user else {
            return false;
        };
        contest.owner_id.is_some_and(|owner_id| owner_id == user.id)
            || contest.co_authors.binary_search(&user.id).is_ok()
            || user.admin_level == crate::models::users::AdminLevel::Owner
    }

    pub fn can_manage_problem(&self, problem: &PublicProblemConfig) -> bool {
        let Some(user) = &self.user else {
            return false;
        };
        problem.owner_id.is_some_and(|owner_id| owner_id == user.id)
            || user.admin_level == crate::models::users::AdminLevel::Owner
    }
}

/// App-wide shared state, provided as a GlobalSignal so any component can read
/// the current language, token and cached data.
pub static STATE: GlobalSignal<GlobalState> = GlobalSignal::new(GlobalState::new);

/// Contest ids with a live ws subscription (dedup across remounts).
pub static WS_SUBSCRIBED: GlobalSignal<std::collections::HashSet<i64>> =
    GlobalSignal::new(std::collections::HashSet::new);

/// Opens an external link in a new browser tab. Mirrors `AJAppCallbacks.open-link`.
pub fn open_link(url: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let _ = window.open_with_url(url);
}

/// Convenience: currently selected UI language ("ru" or "en").
pub fn language() -> String {
    STATE.read().language.clone()
}

/// Convenience: login token, if any.
pub fn token() -> Option<String> {
    STATE.read().token.clone()
}

pub fn save_token(token: &str) {
    let window = web_sys::window();
    let Some(storage) = window.and_then(|w| w.local_storage().ok().flatten()) else {
        return;
    };
    let _ = storage.set_item(TOKEN_KEY, token);
}

pub fn clear_token() {
    let window = web_sys::window();
    let Some(storage) = window.and_then(|w| w.local_storage().ok().flatten()) else {
        return;
    };
    let _ = storage.remove_item(TOKEN_KEY);
}

pub fn load_token() -> Option<String> {
    let window = web_sys::window();
    window
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(TOKEN_KEY).ok().flatten())
}

pub fn save_language(lang: &str) {
    let window = web_sys::window();
    let Some(storage) = window.and_then(|w| w.local_storage().ok().flatten()) else {
        return;
    };
    let _ = storage.set_item(LANG_KEY, lang);
}

pub fn load_language() -> String {
    let window = web_sys::window();
    window
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(LANG_KEY).ok().flatten())
        .filter(|v| v == "ru" || v == "en")
        .unwrap_or_else(|| "ru".to_string())
}

const TAB_KEY: &str = "aj_web_tab";

pub fn save_tab(tab: u8) {
    let window = web_sys::window();
    let Some(storage) = window.and_then(|w| w.local_storage().ok().flatten()) else {
        return;
    };
    let _ = storage.set_item(TAB_KEY, &tab.to_string());
}

pub fn load_tab() -> u8 {
    let window = web_sys::window();
    window
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(TAB_KEY).ok().flatten())
        .and_then(|v| v.parse::<u8>().ok())
        .filter(|v| *v <= 4)
        .unwrap_or(0)
}
use std::fmt;

/// Selected contest tab. Serialized into `?tab=` so the current view can be
/// bookmarked and restored on reload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContestTab {
    #[default]
    Main,
    Submissions,
    Posts,
    Questions,
    Leaderboard,
}

impl ContestTab {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Submissions => "submissions",
            Self::Posts => "posts",
            Self::Questions => "questions",
            Self::Leaderboard => "leaderboard",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "submissions" => Self::Submissions,
            "posts" => Self::Posts,
            "questions" => Self::Questions,
            "leaderboard" => Self::Leaderboard,
            _ => Self::Main,
        }
    }
}

impl fmt::Display for ContestTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Everything the contest page exposes in the URL.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContestQuery {
    pub tab: ContestTab,
    /// 1-based problem number inside the contest.
    pub problem: Option<usize>,
    pub all: bool,
    pub unofficial: bool,
    /// Submission language index: 0 = Rust, 1 = C, 2 = C++, 3 = Go,
    /// 4 = Python, 5 = Pascal.
    pub lang: Option<usize>,
    pub paste: bool,
}

impl From<&str> for ContestQuery {
    fn from(query: &str) -> Self {
        let mut state = Self::default();
        let query = query.strip_prefix('?').unwrap_or(query);

        for pair in query.split('&').filter(|pair| !pair.is_empty()) {
            let Some((key, value)) = pair.split_once('=') else {
                continue;
            };
            match key {
                "tab" => state.tab = ContestTab::parse(value),
                "problem" => {
                    state.problem = value.parse::<usize>().ok().filter(|number| *number > 0);
                }
                "all" => state.all = parse_bool(value),
                "unofficial" => state.unofficial = parse_bool(value),
                "lang" => state.lang = value.parse::<usize>().ok(),
                "paste" => state.paste = parse_bool(value),
                _ => {}
            }
        }

        state
    }
}

impl fmt::Display for ContestQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tab={}", self.tab)?;
        if let Some(problem) = self.problem {
            write!(f, "&problem={problem}")?;
        }
        if self.all {
            f.write_str("&all=1")?;
        }
        if self.unofficial {
            f.write_str("&unofficial=1")?;
        }
        if let Some(lang) = self.lang {
            write!(f, "&lang={lang}")?;
        }
        if self.paste {
            f.write_str("&paste=1")?;
        }
        Ok(())
    }
}

/// Status filter on the contest list. `All` means no filtering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContestFilter {
    #[default]
    All,
    BeforeStart,
    Ongoing,
    Finished,
    Upsolving,
}

impl ContestFilter {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::BeforeStart => "before-start",
            Self::Ongoing => "ongoing",
            Self::Finished => "finished",
            Self::Upsolving => "upsolving",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "before-start" => Self::BeforeStart,
            "ongoing" => Self::Ongoing,
            "finished" => Self::Finished,
            "upsolving" => Self::Upsolving,
            _ => Self::All,
        }
    }

    #[must_use]
    pub const fn status(self) -> Option<crate::state::ContestStatus> {
        match self {
            Self::All => None,
            Self::BeforeStart => Some(crate::state::ContestStatus::BeforeStart),
            Self::Ongoing => Some(crate::state::ContestStatus::Ongoing),
            Self::Finished => Some(crate::state::ContestStatus::Finished),
            Self::Upsolving => Some(crate::state::ContestStatus::Upsolving),
        }
    }
}

/// State of the contest list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomeQuery {
    pub all: bool,
    pub status: ContestFilter,
}

impl Default for HomeQuery {
    fn default() -> Self {
        Self {
            all: true,
            status: ContestFilter::All,
        }
    }
}

impl From<&str> for HomeQuery {
    fn from(query: &str) -> Self {
        let mut state = Self::default();
        let query = query.strip_prefix('?').unwrap_or(query);

        for pair in query.split('&').filter(|pair| !pair.is_empty()) {
            let Some((key, value)) = pair.split_once('=') else {
                continue;
            };
            match key {
                "all" => state.all = parse_bool(value),
                "status" => state.status = ContestFilter::parse(value),
                _ => {}
            }
        }

        state
    }
}

impl fmt::Display for HomeQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "all={}&status={}",
            u8::from(self.all),
            self.status.as_str()
        )
    }
}

/// State of the global problems feed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProblemsQuery {
    pub all: bool,
}

impl From<&str> for ProblemsQuery {
    fn from(query: &str) -> Self {
        let query = query.strip_prefix('?').unwrap_or(query);
        let all = query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .any(|(key, value)| key == "all" && parse_bool(value));
        Self { all }
    }
}

impl fmt::Display for ProblemsQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "all={}", u8::from(self.all))
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(value, "1" | "true" | "yes" | "on")
}

#[cfg(test)]
mod tests {
    use super::{ContestFilter, ContestQuery, ContestTab, HomeQuery, ProblemsQuery};

    #[test]
    fn contest_query_round_trip() {
        let query = ContestQuery {
            tab: ContestTab::Leaderboard,
            problem: Some(3),
            all: true,
            unofficial: true,
            lang: Some(2),
            paste: true,
        };
        let text = query.to_string();
        assert_eq!(
            text,
            "tab=leaderboard&problem=3&all=1&unofficial=1&lang=2&paste=1"
        );
        assert_eq!(ContestQuery::from(text.as_str()), query);
    }

    #[test]
    fn contest_query_defaults_to_main_tab() {
        let query = ContestQuery::from("problem=7");
        assert_eq!(query.tab, ContestTab::Main);
        assert_eq!(query.problem, Some(7));
        assert!(!query.all);
        assert_eq!(ContestQuery::default().to_string(), "tab=main");
    }

    #[test]
    fn home_query_round_trip() {
        let query = HomeQuery {
            all: false,
            status: ContestFilter::Ongoing,
        };
        assert_eq!(query.to_string(), "all=0&status=ongoing");
        assert_eq!(HomeQuery::from(query.to_string().as_str()), query);
        assert!(HomeQuery::default().all);
        assert_eq!(HomeQuery::default().status, ContestFilter::All);
    }

    #[test]
    fn problems_query_round_trip() {
        let query = ProblemsQuery { all: true };
        assert_eq!(query.to_string(), "all=1");
        assert_eq!(ProblemsQuery::from("all=1"), query);
        assert!(!ProblemsQuery::from("all=0").all);
    }
}

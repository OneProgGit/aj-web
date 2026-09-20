use chrono::{DateTime, Datelike, Timelike, Utc};
use dioxus::prelude::*;

/// Browser timezone offset in minutes (UTC − local, as reported by JS).
/// Positive values are west of UTC. Only meaningful in the browser.
fn browser_offset_minutes() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::new_0().get_timezone_offset() as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

/// Formats a UTC DateTime in the browser's local timezone as
/// `dd.MM.yyyy HH:mm:ss`, zero-padded — mirrors aj-app's `AJDateTimeText`.
pub fn format_datetime(dt: DateTime<Utc>) -> String {
    let local = dt + chrono::Duration::minutes(-browser_offset_minutes());
    format!(
        "{:02}.{:02}.{:04} {:02}:{:02}:{:02}",
        local.day(),
        local.month(),
        local.year(),
        local.hour(),
        local.minute(),
        local.second()
    )
}

#[component]
pub fn DateTimeText(time: DateTime<Utc>, #[props(into)] class: Option<String>) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        span { class: class, "{format_datetime(time)}" }
    }
}

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum AlertKind {
    Info,
    Success,
    Warning,
    Error,
}

impl AlertKind {
    #[must_use]
    pub const fn css_class(&self) -> &'static str {
        match self {
            Self::Info => "alert-info",
            Self::Success => "alert-success",
            Self::Warning => "alert-warning",
            Self::Error => "alert-error",
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Info | Self::Success => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alert {
    pub kind: AlertKind,
    pub text: String,
    pub id: u64,
}

/// Globally reachable alert queue.
static ALERTS: GlobalSignal<Vec<Alert>> = GlobalSignal::new(Vec::new);

static NEXT_ID: GlobalSignal<u64> = GlobalSignal::new(Default::default);

/// Последний показанный тост (вид + текст + ms) — давка дублей от двойных
/// кликов и повторных ws-событий.
static LAST_TOAST: GlobalSignal<(AlertKind, String, f64)> =
    GlobalSignal::new(|| (AlertKind::Info, String::new(), 0.0));

pub fn show_alert(kind: AlertKind, text: String) {
    let now = js_sys::Date::now();
    {
        let last = LAST_TOAST.read();
        if last.0 == kind && last.1 == text && now - last.2 < 1500.0 {
            return;
        }
    }
    *LAST_TOAST.write() = (kind, text.clone(), now);
    let id = {
        let mut next = NEXT_ID.write();
        *next += 1;
        *next
    };
    ALERTS.with_mut(|alerts| {
        alerts.push(Alert { kind, text, id });
        while alerts.len() > 5 {
            alerts.remove(0);
        }
    });
    let alerts = &ALERTS;
    gloo_timers::callback::Timeout::new(5_000, move || {
        alerts.write().retain(|alert| alert.id != id);
    })
    .forget();
}

/// daisyUI alert list rendered once at the app root.
#[component]
pub fn AlertHost() -> Element {
    let alerts = ALERTS.read();
    if alerts.is_empty() {
        return rsx! { div {} };
    }
    rsx! {
        div { class: "toast toast-end toast-top z-50",
            for alert in alerts.iter().cloned() {
                div { class: "alert {alert.kind.css_class()} shadow-lg",
                    span { "{alert.text}" }
                    button {
                        class: "btn btn-circle btn-ghost btn-sm",
                        onclick: move |_| { let alerts = &ALERTS; alerts.write().retain(|a| a.id != alert.id); },
                        "✕"
                    }
                }
            }
        }
    }
}

/// Drain the current alerts (used right after login to show a success banner).
#[allow(dead_code)]
pub fn current_alerts() -> Vec<Alert> {
    ALERTS.read().clone()
}
use dioxus::prelude::*;

/// Mirrors aj-app's `AJAlert`. Rendered inline; global queued alerts
/// are shown via `crate::alerts::AlertHost`.
#[component]
pub fn Alert(
    kind: crate::alerts::AlertKind,
    children: Element,
    #[props(into)] class: Option<String>,
) -> Element {
    let class = class.unwrap_or_default();
    rsx! {
        div { role: "alert", class: "alert {kind.css_class()} {class}",
            {children}
        }
    }
}

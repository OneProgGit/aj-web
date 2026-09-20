use dioxus::prelude::*;

use crate::models::users::AdminLevel;

use super::icon::{icon_element, Icon};

/// A person-named button that navigates to a user's profile. Owners are taken
/// to the private profile page, everyone else to the public one (mirrors
/// aj-app's `open-profile` callback).
#[component]
pub fn UserLink(user_id: i64, username: String) -> Element {
    let navigator = use_navigator();
    let is_owner = {
        let state = crate::state::STATE.read();
        state
            .user
            .as_ref()
            .is_some_and(|u| u.admin_level == AdminLevel::Owner)
    };
    rsx! {
        button {
            class: "btn btn-ghost btn-sm gap-1",
            onclick: move |_| {
                if is_owner {
                    navigator.push(crate::Route::UserPrivateProfile { user_id });
                } else {
                    navigator.push(crate::Route::UserProfile { user_id });
                }
            },
            {icon_element(Icon::Person, 14)}
            span { "{username}" }
        }
    }
}

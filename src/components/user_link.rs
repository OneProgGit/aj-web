use dioxus::prelude::*;

use crate::models::users::AdminLevel;

use super::icon::{Icon, icon_slot};

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
        m3e-button {
            variant: "text",
            onclick: move |_| {
                if is_owner {
                    navigator.push(crate::Route::UserPrivateProfile { user_id });
                } else {
                    navigator.push(crate::Route::UserProfile { user_id });
                }
            },
            {icon_slot(Icon::Person, 14)}
            span { "{username}" }
        }
    }
}

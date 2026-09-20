use dioxus::prelude::*;

use crate::models::users::PrivateUserData;

use super::{admin_badge::AdminBadge, user_link::UserLink};

/// A user row in the owner-only users list (mirrors `AJUsersBox`).
#[derive(Props, Clone)]
pub struct UserCardProps {
    pub user: PrivateUserData,
}

impl PartialEq for UserCardProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.user, &other.user)
    }
}

pub fn UserCard(props: UserCardProps) -> Element {
    let UserCardProps { user } = props;
    let lang = crate::state::language();
    rsx! {
        div { class: "card w-full max-w-xl bg-base-200 shadow-lg",
            div { class: "card-body flex-row items-center justify-between",
                div { class: "flex items-center gap-2 flex-1 min-w-0",
                    UserLink { user_id: user.id, username: user.login.clone() }
                    AdminBadge { level: user.admin_level, lang: lang.clone() }
                }
            }
        }
    }
}

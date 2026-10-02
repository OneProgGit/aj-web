use dioxus::prelude::*;

use crate::{
    api,
    components::icon::icon_slot,
    i18n,
    models::{
        DeletionRequest,
        contests::{ContestPost, ContestPostRequest},
    },
    state::STATE,
};

use super::{
    datetime_text::DateTimeText,
    delete_form::DeleteForm,
    icon::Icon,
    markdown::{Markdown, MdField},
    user_link::UserLink,
};

/// Post (announcement) card. Mirrors `AJPostCard`.
#[derive(Props, Clone)]
pub struct PostCardProps {
    pub post: ContestPost,
    pub position: usize,
    pub can_manage: bool,
    pub on_changed: EventHandler<()>,
}

impl PartialEq for PostCardProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.post, &other.post)
            && self.position == other.position
            && self.can_manage == other.can_manage
            && self.on_changed == other.on_changed
    }
}

pub fn PostCard(props: PostCardProps) -> Element {
    let PostCardProps {
        post,
        position,
        can_manage,
        on_changed,
    } = props;
    let lang = crate::state::language();
    let mut editing = use_signal(|| false);
    let mut deleting = use_signal(|| false);

    let title = if lang == "en" && !post.title_en.is_empty() {
        post.title_en.clone()
    } else {
        post.title_ru.clone()
    };
    let text = if lang == "en" && !post.text_en.is_empty() {
        post.text_en.clone()
    } else {
        post.text_ru.clone()
    };

    rsx! {
        div { class: "card w-full max-w-3xl bg-base-200 shadow-lg",
            div { class: "card-body gap-3",
                div { class: "flex items-center justify-between gap-4",
                    span { class: "text-sm", "{i18n::tr(&lang, \"автор\", \"author\")}" }
                    UserLink { user_id: post.owner_id, username: post.owner_login.clone() }
                }
                div { class: "flex items-center justify-between gap-4",
                    span { class: "text-sm", "{i18n::tr(&lang, \"создано\", \"created\")}" }
                    DateTimeText { time: post.created_at, class: "text-sm italic text-right" }
                }
                h3 { class: "card-title", span { "#{position} {title}" } }
                Markdown { text: text }

                if can_manage {
                    div { class: "flex items-center gap-2",
                        m3e-button {
                            variant: "text",
                            onclick: move |_| editing.set(!editing()),
                            {icon_slot(Icon::Pencil, 16)}
                            span { "{i18n::tr(&lang, \"изменить\", \"edit\")}" }
                        }
                        m3e-button {
                            class: "error-text",
                            variant: "text",
                            onclick: move |_| deleting.set(!deleting()),
                            {icon_slot(Icon::Trash, 16)}
                            span { "{i18n::tr(&lang, \"удалить\", \"delete\")}" }
                        }
                    }

                    if editing() {
                        div { class: "modal modal-open",
                            div { class: "modal-box max-w-2xl",
                                div { class: "flex items-center justify-between",
                                    h3 { class: "card-title", "{i18n::tr(&lang, \"Изменить объявление\", \"Edit announcement\")}" }
                                    m3e-icon-button {
                                        onclick: move |_| editing.set(false),
                                        "✕"
                                    }
                                }
                                {post_edit_form(post.clone(), Callback::new(move |request| {
                                    let token = STATE.read().token.clone();
                                    let id = post.id;
                                    spawn(async move {
                                        match api::contests::update_contest_post(id, &request, &token).await {
                                            Ok(()) => {
                                                editing.set(false);
                                                on_changed.call(());
                                            }
                                            Err(e) => crate::alerts::show_alert(crate::alerts::AlertKind::Error, e),
                                        }
                                    });
                                }), Callback::new(move |_| editing.set(false)))}
                            }
                        }
                    }

                    if deleting() {
                        DeleteForm {
                            on_delete: move |(login, password, confirm)| {
                                let token = STATE.read().token.clone();
                                let id = post.id;
                                let request = DeletionRequest { login, password, deletion_confirmation: confirm };
                                spawn(async move {
                                    match api::contests::delete_contest_post(id, &request, &token).await {
                                        Ok(()) => {
                                            deleting.set(false);
                                            on_changed.call(());
                                        }
                                        Err(e) => crate::alerts::show_alert(crate::alerts::AlertKind::Error, e),
                                    }
                                });
                            },
                            on_cancel: move |_| deleting.set(false),
                        }
                    }
                }
            }
        }
    }
}

fn post_edit_form(
    post: ContestPost,
    on_submit: EventHandler<ContestPostRequest>,
    on_cancel: Callback<MouseEvent>,
) -> Element {
    let lang = crate::state::language();
    let mut title_ru = use_signal(|| post.title_ru.clone());
    let text_ru = use_signal(|| post.text_ru.clone());
    let mut title_en = use_signal(|| post.title_en.clone());
    let text_en = use_signal(|| post.text_en.clone());
    let valid = (!title_ru().is_empty() && !text_ru().is_empty())
        || (!title_en().is_empty() && !text_en().is_empty());

    rsx! {
        div { class: "flex flex-col gap-3",
            m3e-form-field {
            span { slot: "label", "{i18n::tr(&lang, \"название (рус.)\", \"title (ru)\")}" }
                input {
                    value: title_ru(),
                    oninput: move |ev| title_ru.set(ev.value()),
                }
            }
            MdField {
                value: text_ru,
                label: i18n::tr(&lang, "текст (рус.)", "text (ru)"),
            }
            m3e-form-field {
            span { slot: "label", "{i18n::tr(&lang, \"название (англ.)\", \"title (en)\")}" }
                input {
                    value: title_en(),
                    oninput: move |ev| title_en.set(ev.value()),
                }
            }
            MdField {
                value: text_en,
                label: i18n::tr(&lang, "текст (англ.)", "text (en)"),
            }
            div { class: "card-actions justify-end",
                m3e-button {
                    variant: "text",
                    onclick: move |ev| on_cancel.call(ev),
                    span { "{i18n::tr(&lang, \"отменить\", \"cancel\")}" }
                }
                m3e-button {
                    variant: "filled",
                    disabled: !valid,
                    onclick: move |_| {
                        on_submit.call(ContestPostRequest {
                            title_ru: title_ru(), text_ru: text_ru(),
                            title_en: title_en(), text_en: text_en(),
                        });
                    },
                    {icon_slot(if valid { Icon::Pencil } else { Icon::CircleBackslash }, 16)}
                    span { "{i18n::tr(&lang, \"изменить\", \"edit\")}" }
                }
            }
        }
    }
}

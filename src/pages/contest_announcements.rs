use dioxus::prelude::*;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        icon::{icon_element, Icon},
        markdown::MdField,
        post_card::PostCard,
    },
    i18n,
    models::contests::ContestPostRequest,
    state::STATE,
};

#[component]
pub fn ContestAnnouncements(contest_id: i64) -> Element {
    let lang = crate::state::language();
    let mut modal_open = use_signal(|| false);
    let mut title_ru = use_signal(String::new);
    let text_ru = use_signal(String::new);
    let mut title_en = use_signal(String::new);
    let text_en = use_signal(String::new);
    let is_owner = STATE.read().is_owner();

    let reload = move || {
        let token = crate::state::token();
        spawn(async move {
            match api::contests::get_contest_posts(contest_id, &token).await {
                Ok(posts) => STATE.write().posts = posts,
                Err(e) => show_alert(AlertKind::Error, e),
            }
        });
    };

    let valid = (!title_ru().is_empty() && !text_ru().is_empty())
        || (!title_en().is_empty() && !text_en().is_empty());

    rsx! {
        div { class: "flex flex-col gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex flex-wrap gap-4 items-center",
                h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Объявления в контесте\", \"Contest announcements\")}" }

                button {
                    class: "btn btn-neutral btn-sm gap-1",
                    onclick: move |_| modal_open.set(true),
                    {icon_element(Icon::Plus, 16)}
                    span { "{i18n::tr(&lang, \"создать\", \"create\")}" }
                }
            }

            if STATE.read().posts.is_empty() {
                p { class: "italic", "{i18n::tr(&lang, \"Объявлений пока что нет\", \"No announcements yet\")}" }
            } else {
                div { class: "flex flex-col gap-4 w-full max-h-[32rem] overflow-y-auto",
                    for (i, post) in STATE.read().posts.iter().enumerate() {
                        PostCard {
                            post: post.clone(),
                            position: STATE.read().posts.len() - i,
                            can_manage: post.owner_id == STATE.read().user.as_ref().map(|u| u.id).unwrap_or_default() || is_owner,
                            on_changed: {
                                move |_| reload()
                            },
                        }
                    }
                }
            }

            if modal_open() {
                div { class: "modal modal-open",
                    div { class: "modal-box max-w-2xl",
                        div { class: "flex items-center justify-between",
                            h3 { class: "card-title", "{i18n::tr(&lang, \"Создать объявление\", \"Create post\")}" }
                            button { class: "btn btn-sm btn-circle btn-ghost", onclick: move |_| modal_open.set(false), "✕" }
                        }
                        div { class: "flex flex-col gap-3 mt-4",
                            span { class: "label-text", "{i18n::tr(&lang, \"название (рус.)\", \"title (ru)\")}" }
                            input {
                                class: "input input-bordered",
                                value: title_ru(),
                                oninput: move |ev| title_ru.set(ev.value()),
                            }
                            MdField {
                                value: text_ru,
                                label: i18n::tr(&lang, "текст (рус.)", "text (ru)"),
                            }
                            span { class: "label-text", "{i18n::tr(&lang, \"название (англ.)\", \"title (en)\")}" }
                            input {
                                class: "input input-bordered",
                                value: title_en(),
                                oninput: move |ev| title_en.set(ev.value()),
                            }
                            MdField {
                                value: text_en,
                                label: i18n::tr(&lang, "текст (англ.)", "text (en)"),
                            }
                            div { class: "card-actions justify-end",
                                button {
                                    class: "btn btn-ghost btn-sm gap-1",
                                    onclick: move |_| modal_open.set(false),
                                    span { "{i18n::tr(&lang, \"отменить\", \"cancel\")}" }
                                }
                                button {
                                    class: "btn btn-primary btn-sm gap-1",
                                    disabled: !valid,
                                    onclick: {
                                        move |_| {
                                            let request = ContestPostRequest {
                                                title_ru: title_ru(),
                                                text_ru: text_ru(),
                                                title_en: title_en(),
                                                text_en: text_en(),
                                            };
                                            let token = crate::state::token();
                                            let mut modal = modal_open;
                                            spawn(async move {
                                                match api::contests::create_contest_post(contest_id, &request, &token).await {
                                                    Ok(()) => {
                                                        modal.set(false);
                                                        reload();
                                                    }
                                                    Err(e) => show_alert(AlertKind::Error, e),
                                                }
                                            });
                                        }
                                    },
                                    {icon_element(if valid { Icon::Plus } else { Icon::CircleBackslash }, 16)}
                                    span { "{i18n::tr(&lang, \"создать\", \"create\")}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

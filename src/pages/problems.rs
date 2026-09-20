use dioxus::prelude::*;
use dioxus_web::WebEventExt;
use wasm_bindgen::JsCast;

use crate::{
    alerts::{show_alert, AlertKind},
    api,
    components::{
        icon::{icon_element, Icon},
        problem_card::ProblemCard,
    },
    i18n,
    state::STATE,
};

#[component]
pub fn Problems() -> Element {
    let lang = crate::state::language();
    let navigator = use_navigator();
    let mut all_problems = use_signal(|| false);
    let mut picked = use_signal(|| None::<(String, Vec<u8>)>);
    let mut loaded = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let is_owner = STATE.read().is_owner();

    if !*loaded.read() {
        loaded.set(true);
        STATE.write().problems = Vec::new();
        let all = all_problems();
        let token = crate::state::token();
        spawn(async move {
            let res = if all {
                api::problems::get_all_problems(&token).await
            } else {
                api::problems::get_my_problems(&token).await
            };
            match res {
                Ok(list) => STATE.write().problems = list,
                Err(e) => show_alert(AlertKind::Error, e),
            }
        });
    }

    let pick_label = if let Some((name, _)) = picked() {
        i18n::tr(
            &lang,
            &format!("выбрать архив ({name})"),
            &format!("pick archive ({name})"),
        )
    } else {
        i18n::tr(
            &lang,
            "выбрать архив",
            "pick archive",
        )
    };

    rsx! {
        div { class: "flex flex-col gap-4 max-w-7xl mx-auto w-full",
            div { class: "flex flex-wrap gap-4 items-center",
                button {
                    class: "btn btn-ghost btn-sm gap-2",
                    onclick: move |_| { let _ = navigator.push(crate::Route::Home {}); },
                    {icon_element(Icon::Back, 16)}
                    span { "{i18n::tr(&lang, \"назад\", \"back\")}" }
                }
                h1 { class: "text-2xl font-bold", "{i18n::tr(&lang, \"Задачи\", \"Problems\")}" }

                label { class: "btn btn-ghost btn-sm gap-1",
                    {icon_element(if picked().is_some() { Icon::Pencil } else { Icon::Upload }, 16)}
                    span { "{pick_label}" }
                    input {
                        r#type: "file",
                        class: "hidden",
                        onchange: move |ev| {
                            let input = ev
                                .as_web_event()
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                            let file = input
                                .as_ref()
                                .and_then(|el| el.files())
                                .and_then(|f| f.item(0));
                            if let Some(file) = file {
                                let name = file.name();
                                spawn(async move {
                                    let buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await;
                                    match buffer {
                                        Ok(b) => {
                                            let view = js_sys::Uint8Array::new(&b);
                                            picked.set(Some((name, view.to_vec())));
                                        }
                                        Err(_) => show_alert(AlertKind::Error, crate::i18n::tr(&crate::state::language(), "Не удалось прочитать файл", "Could not read file")),
                                    }
                                });
                            }
                            if let Some(input) = input {
                                input.set_value("");
                            }
                        },
                    }
                }

                button {
                    class: "btn btn-neutral btn-sm gap-1",
                    disabled: busy() || picked().is_none(),
                    onclick: {
                        move |_| {
                            let Some((_, bytes)) = picked().clone() else { return; };
                            if busy() {
                                return;
                            }
                            busy.set(true);
                            let token = crate::state::token();
                            spawn(async move {
                                match api::problems::create_problem(bytes, &token).await {
                                    Ok(()) => {
                                        busy.set(false);
                                        let all = all_problems();
                                        let res = if all {
                                            api::problems::get_all_problems(&token).await
                                        } else {
                                            api::problems::get_my_problems(&token).await
                                        };
                                        if let Ok(list) = res {
                                            STATE.write().problems = list;
                                        }
                                    }
                                    Err(e) => {
                                        busy.set(false);
                                        show_alert(AlertKind::Error, e);
                                    }
                                }
                            });
                        }
                    },
                    {icon_element(if picked().is_none() { Icon::CircleBackslash } else { Icon::Plus }, 16)}
                    span { "{i18n::tr(&lang, \"создать\", \"create\")}" }
                }
            }

            if is_owner {
                label { class: "label cursor-pointer justify-start gap-2",
                    input {
                        r#type: "checkbox",
                        class: "checkbox checkbox-sm",
                        checked: all_problems(),
                        onchange: move |ev| {
                            let new_value = ev.checked();
                            all_problems.set(new_value);
                            let token = crate::state::token();
                            spawn(async move {
                                let res = if new_value {
                                    api::problems::get_all_problems(&token).await
                                } else {
                                    api::problems::get_my_problems(&token).await
                                };
                                match res {
                                    Ok(list) => STATE.write().problems = list,
                                    Err(e) => show_alert(AlertKind::Error, e),
                                }
                            });
                        },
                    }
                    span { class: "label-text", "{i18n::tr(&lang, \"все задачи\", \"all problems\")}" }
                }
            }

            if STATE.read().problems.is_empty() {
                p { class: "italic", "{i18n::tr(&lang, \"Задач пока что нет\", \"No problems yet\")}" }
            } else {
                div { class: "flex flex-col gap-4 w-full max-h-[32rem] overflow-y-auto",
                    for p in STATE.read().problems.iter() {
                        div {
                            ProblemCard {
                                problem: p.clone(),
                                show_id: true,
                                subgroups_scroll: true,
                                on_changed: move |_| {},
                            }
                        }
                    }
                }
            }
        }
    }
}

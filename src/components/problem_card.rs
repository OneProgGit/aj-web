use dioxus::prelude::*;
use dioxus_web::WebEventExt;
use wasm_bindgen::JsCast;

use crate::{
    api, i18n,
    models::{
        problems::{ProblemType, PublicProblemConfig, Subgroup, SubgroupType},
        DeletionRequest,
    },
    state::STATE,
};

use super::{
    delete_form::DeleteForm,
    icon::{icon_element, Icon},
};

/// Subgroup mini-card inside a problem card (mirrors `AJSubgroupCard`).
#[derive(Props, Clone)]
pub struct SubgroupCardProps {
    pub subgroup: Subgroup,
    pub index: usize,
}

impl PartialEq for SubgroupCardProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.subgroup, &other.subgroup)
            && self.index == other.index
    }
}

pub fn SubgroupCard(props: SubgroupCardProps) -> Element {
    let SubgroupCardProps { subgroup, index } = props;
    let lang = crate::state::language();
    rsx! {
        div { class: "rounded-lg bg-base-300 p-3 flex flex-col gap-2 text-base-content",
            p { class: "font-medium", "##{index}" }
            div { class: "flex justify-between",
                span { class: "text-sm", "{i18n::tr(&lang, \"тип\", \"type\")}" }
                span { class: "text-sm italic",
                    {match subgroup.r#type {
                        SubgroupType::Main => i18n::tr(&lang, "основная", "main"),
                        SubgroupType::Sample => i18n::tr(&lang, "примеры", "samples"),
                    }}
                }
            }
            div { class: "flex justify-between gap-4",
                span { class: "text-sm shrink-0", "{i18n::tr(&lang, \"тесты\", \"tests\")}" }
                span { class: "text-sm italic text-right min-w-0 break-words",
                    "{subgroup.tests.iter().map(i32::to_string).collect::<Vec<_>>().join(\", \")}"
                }
            }
            div { class: "flex justify-between gap-4",
                span { class: "text-sm shrink-0", "{i18n::tr(&lang, \"требуемые подгруппы\", \"required subgroups\")}" }
                span { class: "text-sm italic text-right min-w-0 break-words",
                    {if subgroup.depends_on.is_empty() {
                        "-".to_string()
                    } else {
                        subgroup.depends_on.iter().map(usize::to_string).collect::<Vec<_>>().join(", ")
                    }}
                }
            }
            if subgroup.score.is_some() && subgroup.r#type == SubgroupType::Main {
                div { class: "flex justify-between",
                    span { class: "text-sm", "{i18n::tr(&lang, \"баллы\", \"score\")}" }
                    span { class: "text-sm italic", "{subgroup.score.unwrap_or_default()}" }
                }
            }
            if subgroup.score_per_test.is_some() {
                div { class: "flex justify-between",
                    span { class: "text-sm", "{i18n::tr(&lang, \"баллы за тест\", \"score per test\")}" }
                    span { class: "text-sm italic", "{subgroup.score_per_test.unwrap_or_default()}" }
                }
            }
        }
    }
}

/// Problem card. Mirrors `AJProblemCard`.
#[derive(Props, Clone)]
pub struct ProblemCardProps {
    pub problem: PublicProblemConfig,
    pub show_id: bool,
    pub on_changed: EventHandler<()>,
    pub subgroups_scroll: bool,
}

impl PartialEq for ProblemCardProps {
    fn eq(&self, other: &Self) -> bool {
        crate::models::props_json_eq(&self.problem, &other.problem)
            && self.show_id == other.show_id
            && self.on_changed == other.on_changed
            && self.subgroups_scroll == other.subgroups_scroll
    }
}

pub fn ProblemCard(props: ProblemCardProps) -> Element {
    let ProblemCardProps {
        problem,
        show_id,
        on_changed,
        subgroups_scroll,
    } = props;
    let lang = crate::state::language();
    let can_manage = STATE.read().can_manage_problem(&problem);
    let mut admin_open = use_signal(|| false);
    let mut deleting = use_signal(|| false);
    let mut picked_archive = use_signal(|| None::<(String, Vec<u8>)>);

    let name = if lang == "en" && !problem.name_en.is_empty() {
        problem.name_en.clone()
    } else {
        problem.name_ru.clone()
    };

    rsx! {
        div { class: "card w-full max-w-3xl flex-1 min-h-0 min-w-0 bg-base-200 shadow-lg overflow-y-auto",
            div { class: "card-body gap-3 min-w-0",
                div { class: "flex items-center justify-between gap-2",
                    h3 { class: "card-title text-base",
                        span {
                            {if show_id {
                                format!("#{} {name}", problem.id)
                            } else {
                                format!("#{} {name}", problem.index + 1)
                            }}
                        }
                    }
                    if can_manage {
                        button {
                            class: "btn btn-ghost btn-sm gap-1",
                            onclick: move |_| admin_open.set(!admin_open()),
                            {icon_element(Icon::Gear, 14)}
                            span { "{i18n::tr(&lang, \"админ.\", \"admin\")}" }
                        }
                    }
                }

                if matches!(problem.r#type, ProblemType::Interactive) {
                    p { class: "font-bold", "{i18n::tr(&lang, \"Это интерактивная задача.\", \"This is an interactive problem.\")}" }
                }
                if matches!(problem.r#type, ProblemType::RunTwice) {
                    p { class: "font-bold", "{i18n::tr(&lang, \"Это задача с двойным запуском.\", \"This problem is run twice.\")}" }
                }
                if matches!(problem.r#type, ProblemType::RunTwiceFirstInteractive | ProblemType::RunTwiceSecondInteractive) {
                    p { class: "font-bold", "{i18n::tr(&lang, \"Это интерактивная задача с двойным запуском.\", \"This is an interactive problem run twice.\")}" }
                }

                div { class: "flex justify-between",
                    span { class: "text-sm", "{i18n::tr(&lang, \"ограничение по времени\", \"time limit\")}" }
                    span { class: "text-sm italic", "{problem.time_limit_ms}мс" }
                }
                div { class: "flex justify-between",
                    span { class: "text-sm", "{i18n::tr(&lang, \"ограничение по памяти\", \"memory limit\")}" }
                    span { class: "text-sm italic", "{problem.memory_limit_mb}МБ" }
                }

                p { class: "font-medium", "{i18n::tr(&lang, \"Подгруппы\", \"Subgroups\")}" }
                div { class: if subgroups_scroll { "flex flex-col gap-2 max-h-96 overflow-y-auto" } else { "flex flex-col gap-2" },
                    for (i, s) in problem.subgroups.iter().enumerate() {
                        SubgroupCard { subgroup: s.clone(), index: i }
                    }
                }

                if admin_open() {
                    div { class: "modal modal-open",
                        div { class: "modal-box max-w-2xl flex flex-col gap-3",
                            div { class: "flex justify-between items-center",
                                span { class: "text-lg font-bold", "{i18n::tr(&lang, \"Управление задачей\", \"Problem management\")}" }
                                button {
                                    class: "btn btn-sm btn-circle btn-ghost",
                                    onclick: move |_| { picked_archive.set(None); admin_open.set(false); },
                                    "✕"
                                }
                            }
                            label { class: "btn btn-block btn-outline btn-sm gap-1",
                            {icon_element(Icon::Upload, 16)}
                            span {
                                {if let Some((name, _)) = picked_archive() {
                                    format!("{} ({})", i18n::tr(&lang, "выбрать архив", "pick archive"), name)
                                } else {
                                    i18n::tr(&lang, "выбрать архив", "pick archive")
                                }}
                            }
                            input {
                                r#type: "file",
                                accept: ".zip,.tgz,.tar.gz",
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
                                                    picked_archive.set(Some((name, view.to_vec())));
                                                }
                                                Err(_) => crate::alerts::show_alert(crate::alerts::AlertKind::Error, crate::i18n::tr(&crate::state::language(), "Не удалось прочитать файл", "Could not read file")),
                                            }
                                        });
                                    }
                                    if let Some(input) = input {
                                        input.set_value("");
                                    }
                                },
                            }
                        }
                        div { class: "card-actions justify-end mt-2",
                            button {
                                class: "btn btn-ghost btn-sm gap-1",
                                onclick: move |_| { picked_archive.set(None); admin_open.set(false); },
                                span { "{i18n::tr(&lang, \"отменить\", \"cancel\")}" }
                            }
                            button {
                                class: "btn btn-accent btn-sm gap-1",
                                onclick: {
                                    let token = STATE.read().token.clone();
                                    let pid = problem.id;
                                    move |_| {
                                        let token = token.clone();
                                        spawn(async move {
                                            if let Err(e) = api::problems::download_problem(pid, &token).await {
                                                crate::alerts::show_alert(crate::alerts::AlertKind::Error, e);
                                            }
                                        });
                                    }
                                },
                                {icon_element(Icon::Download, 16)}
                                span { "{i18n::tr(&lang, \"скачать\", \"download\")}" }
                            }
                            button {
                                class: "btn btn-error btn-sm gap-1",
                                onclick: move |_| deleting.set(!deleting()),
                                {icon_element(Icon::Trash, 16)}
                                span { "{i18n::tr(&lang, \"удалить\", \"delete\")}" }
                            }
                            button {
                            class: "btn btn-primary btn-sm gap-1",
                            disabled: picked_archive().is_none(),
                            onclick: {
                                let archive = picked_archive().map(|(_, b)| b);
                                let token = STATE.read().token.clone();
                                let pid = problem.id;
                                move |_| {
                                    if let Some(bytes) = archive.clone() {
                                        let token = token.clone();
                                        spawn(async move {
                                            match api::problems::update_problem(pid, bytes, &token).await {
                                                Ok(()) => {
                                                    picked_archive.set(None);
                                                    admin_open.set(false);
                                                    on_changed.call(());
                                                }
                                                Err(e) => crate::alerts::show_alert(crate::alerts::AlertKind::Error, e),
                                            }
                                        });
                                    }
                                }
                            },
                            {icon_element(Icon::Pencil, 16)}
                            span { "{i18n::tr(&lang, \"изменить\", \"edit\")}" }
                            }
                        }
                    }
                }

                if deleting() {
                    DeleteForm {
                        on_delete: move |(login, password, confirm)| {
                            let token = STATE.read().token.clone();
                            let pid = problem.id;
                            let request = DeletionRequest { login, password, deletion_confirmation: confirm };
                            spawn(async move {
                                match api::problems::delete_problem(pid, &request, &token).await {
                                    Ok(()) => {
                                        deleting.set(false);
                                        admin_open.set(false);
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

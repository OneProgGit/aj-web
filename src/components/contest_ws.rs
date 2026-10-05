use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use chrono::Utc;

use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::ws::{subscribe_ws, unsubscribe_ws};

use crate::{
    alerts::{AlertKind, show_alert},
    i18n,
    models::contests::ContestsEvent,
    state::{ContestStatus, STATE, contest_status},
};

/// Вставка/замена с сохранением серверного порядка (`c.id desc`):
/// существующий — заменяется на месте (id неизменен, порядок цел),
/// отсутствующий (новый, отжатый "скрыть") — встаёт по своему id.
fn upsert_contest(
    list: &mut Vec<crate::models::contests::PublicContestConfig>,
    fresh: crate::models::contests::PublicContestConfig,
) {
    if let Some(slot) = list.iter_mut().find(|c| c.id == fresh.id) {
        *slot = fresh;
        return;
    }
    let pos = list
        .iter()
        .position(|c| fresh.id > c.id)
        .unwrap_or(list.len());
    list.insert(pos, fresh);
}

/// Полный GET контеста. Возвращает true, если контест виден пользователю.
/// На 403 (скрыли/исключили) — тихо выкидываем из списка и отписываемся,
/// без уведомлений: для нас контест просто исчез.
async fn refresh_contest(id: i64) -> bool {
    let token = crate::state::token();
    match crate::api::contests::get_contest(id, &token).await {
        Ok(fresh) => {
            upsert_contest(&mut STATE.write().contests, fresh);
            true
        }
        Err(e) => {
            if e.contains("Forbidden") || e.contains("Доступ запрещён") {
                STATE.write().contests.retain(|c| c.id != id);
                crate::components::ws::unsubscribe_ws(&contest_key(id));
            }
            false
        }
    }
}

async fn reload_posts(contest_id: i64) {
    let token = crate::state::token();
    if let Ok(posts) = crate::api::contests::get_contest_posts(contest_id, &token).await {
        STATE.write().posts = posts;
    }
}

async fn reload_problems(contest_id: i64) {
    let token = crate::state::token();
    if let Ok(list) = crate::api::problems::get_contest_problems(contest_id, &token).await {
        STATE.write().contest_problems = list;
    }
}

async fn reload_questions(contest_id: i64) {
    let token = crate::state::token();
    let can_manage = STATE
        .read()
        .contests
        .iter()
        .find(|c| c.id == contest_id)
        .is_some_and(|c| STATE.read().can_manage_contest(c));
    let res = if can_manage {
        crate::api::contests::get_contest_questions_all(contest_id, &token).await
    } else {
        crate::api::contests::get_contest_questions_my(contest_id, &token).await
    };
    if let Ok(questions) = res {
        STATE.write().questions = questions;
    }
}

/// Прошлый известный статус каждого контеста: по нему видно, что контест
/// только что стартовал или закончился.
static LAST_STATUS: Mutex<Option<HashMap<i64, ContestStatus>>> = Mutex::new(None);

/// Перечитывает контест, у которого сменился статус.
///
/// События «контест начался/закончился» в ленте нет, а сам статус выводится
/// из `finishes_at` против `Utc::now()` и меняется без нашего участия. Поэтому
/// сверяемся с предыдущим снимком и при расхождении делаем GET:
///
/// - на старте — чтобы появились условия (они закрыты до начала);
/// - на финише — чтобы появился разбор при включённой дорешке.
///
/// Вызывается из heartbeat-тика `ws_loop`, поэтому работает и на странице
/// списка (лента), и на странице контеста (его сокет). Дублировать запросы
/// не выйдет: снимок обновляется сразу, и второй сокет расхождений не увидит.
async fn refresh_contests_on_status_change() {
    let now: Vec<(i64, ContestStatus)> = STATE
        .read()
        .contests
        .iter()
        .map(|c| (c.id, contest_status(c)))
        .collect();

    let changed: Vec<(i64, ContestStatus)> = {
        let Ok(mut guard) = LAST_STATUS.lock() else {
            return;
        };
        let Some(snapshot) = guard.as_ref() else {
            // Первый вызов: состояние только что загрузили, перечитывать нечего.
            // Запоминаем снимок, с которого будем сравнивать дальше.
            *guard = Some(now.into_iter().collect());
            return;
        };
        let changed: Vec<(i64, ContestStatus)> = now
            .into_iter()
            .filter(|(id, status)| snapshot.get(id) != Some(status))
            .collect();
        // Снимок обновляем сразу, до await: иначе два сокета (страница контеста и
        // лента списка) за один тик запросят одно и то же дважды.
        *guard = Some(
            STATE
                .read()
                .contests
                .iter()
                .map(|c| (c.id, contest_status(c)))
                .collect(),
        );
        changed
    };
    if changed.is_empty() {
        return;
    }

    let known: HashSet<i64> = STATE.read().contests.iter().map(|c| c.id).collect();
    {
        let Ok(mut guard) = LAST_STATUS.lock() else {
            return;
        };
        if let Some(map) = guard.as_mut() {
            // Отсекаем протухшие id, чтобы снимок не рос бесконечно.
            map.retain(|id, _| known.contains(id));
        }
    }

    for (id, status) in changed {
        let started = status == ContestStatus::Ongoing;
        refresh_contest(id).await;
        // Условия перечитываем только для открытого контеста: STATE.contest_problems
        // принадлежит той странице, чей сокет открыт. У неё же мы и подписаны.
        if started && crate::components::ws::is_subscribed(&contest_key(id)) {
            reload_problems(id).await;
        }
    }
}

/// Следит за сменой статуса контестов и перечитывает их в момент перехода.
///
/// Статус выводится из времени (`contest_status`), то есть меняется ровно в
/// `starts_at` / `finishes_at`. Карточка в списке обновляет его раз в секунду
/// по своему таймеру, но payload контеста (условия на старте, разбор на
/// финише) без GET остаётся старым — поэтому и сам GET ставим точно в момент
/// перехода, а не по круговому опросу: реакция в пределах секунды.
///
/// Ждём не дольше минуты: за это время список контестов может обновиться
/// (по WS) и добавиться контест с более ранним переходом.
pub fn contest_status_watcher() {
    static STARTED: OnceLock<()> = OnceLock::new();
    if STARTED.set(()).is_err() {
        return;
    }
    spawn_local(async move {
        const MAX_WAIT_MS: i64 = 60_000;
        loop {
            let now = Utc::now();
            let next = STATE
                .read()
                .contests
                .iter()
                .flat_map(|c| [c.starts_at, c.finishes_at])
                .filter(|t| *t > now)
                .min();
            let wait = match next {
                // +1s: в момент finishes_at сервер может ещё отдавать контест
                // как незавершённый.
                Some(t) => (t - now).num_milliseconds().clamp(0, MAX_WAIT_MS) + 1_000,
                None => MAX_WAIT_MS,
            };
            gloo_timers::future::TimeoutFuture::new(wait.min(MAX_WAIT_MS) as u32).await;
            refresh_contests_on_status_change().await;
        }
    });
}

/// Открывает ws на /contests/{id}/ws и на каждое событие обновляет
/// глобальное состояние.
pub fn contest_ws(contest_id: i64) {
    subscribe_ws(
        contest_key(contest_id),
        format!("/contests/{contest_id}/ws"),
        move |event| handle_event(Some(contest_id), event),
    );
}

/// Ключ подписки на контест: он же используется для отписки.
#[must_use]
pub fn contest_key(contest_id: i64) -> String {
    format!("contest:{contest_id}")
}

/// Подписка списка контестов: все (`/contests/ws`) или свои (`/contests/my/ws`).
/// Обрабатывает только членство списка (создание/обновление/удаление),
/// чтобы не спамить тостами постов чужих контестов.
pub fn contests_feed_ws(mine: bool) {
    let (key, path) = if mine {
        ("contests:my".to_string(), "/contests/my/ws")
    } else {
        ("contests:all".to_string(), "/contests/ws")
    };
    subscribe_ws(key, path.to_string(), handle_feed_event);
}

async fn handle_feed_event(event: ContestsEvent) {
    handle_event(None, event).await;
}

async fn handle_event(contest_id: Option<i64>, event: ContestsEvent) {
    let lang = crate::state::language();
    // В фиде (контекста контеста нет) — только членство списка.
    let notice = match &event {
        ContestsEvent::NewPost(id) => {
            if let Some(cid) = contest_id {
                reload_posts(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Новое объявление #{id}"),
                    &format!("New announcement #{id}"),
                )
            })
        }
        ContestsEvent::PostUpdated(id) => {
            if let Some(cid) = contest_id {
                reload_posts(cid).await;
                // Номер как в карточках: позиция с конца списка.
                let state = STATE.read();
                let n = state
                    .posts
                    .iter()
                    .position(|p| p.id == *id)
                    .map(|i| (state.posts.len() - i) as i64)
                    .unwrap_or(*id);
                Some(i18n::tr(
                    &lang,
                    &format!("Объявление #{n} обновлено"),
                    &format!("Announcement #{n} updated"),
                ))
            } else {
                None
            }
        }
        ContestsEvent::PostDeleted(id) => {
            if let Some(cid) = contest_id {
                // Индекс берём до перезагрузки — после неё поста уже нет.
                let n = {
                    let state = STATE.read();
                    state
                        .posts
                        .iter()
                        .position(|p| p.id == *id)
                        .map(|i| (state.posts.len() - i) as i64)
                        .unwrap_or(*id)
                };
                reload_posts(cid).await;
                Some(i18n::tr(
                    &lang,
                    &format!("Объявление #{n} удалено"),
                    &format!("Announcement #{n} deleted"),
                ))
            } else {
                None
            }
        }
        ContestsEvent::ContestUpdated(id) => {
            // Тост только если контест нам виден; скрытый тихо исчезает из списка.
            if refresh_contest(*id).await {
                Some(i18n::tr(
                    &lang,
                    &format!("Контест #{id} обновлён"),
                    &format!("Contest #{id} updated"),
                ))
            } else {
                None
            }
        }
        ContestsEvent::ContestDeleted(id) => {
            // Тост только если контест был у нас в списке; чужой/скрытый — тихо.
            let mut state = STATE.write();
            let had = state.contests.iter().any(|c| c.id == *id);
            state.contests.retain(|c| c.id != *id);
            drop(state);
            unsubscribe_ws(&contest_key(*id));
            had.then(|| {
                i18n::tr(
                    &lang,
                    &format!("Контест #{id} удалён"),
                    &format!("Contest #{id} deleted"),
                )
            })
        }
        ContestsEvent::NewContest(id) => {
            // Тост только если контест нам виден (скрытый от нас — тихо).
            let token = crate::state::token();
            match crate::api::contests::get_contest(*id, &token).await {
                Ok(fresh) => {
                    upsert_contest(&mut STATE.write().contests, fresh);
                    Some(i18n::tr(
                        &lang,
                        &format!("Контест #{id} создан"),
                        &format!("Contest #{id} created"),
                    ))
                }
                Err(_) => None,
            }
        }
        ContestsEvent::NewProblem(id) => {
            if let Some(cid) = contest_id {
                reload_problems(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Новая задача #{id}"),
                    &format!("New problem #{id}"),
                )
            })
        }
        ContestsEvent::ProblemUpdated(id) => {
            if let Some(cid) = contest_id {
                reload_problems(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Задача #{id} обновлена"),
                    &format!("Problem #{id} updated"),
                )
            })
        }
        ContestsEvent::ProblemDeleted(id) => {
            if let Some(cid) = contest_id {
                reload_problems(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Задача #{id} удалена"),
                    &format!("Problem #{id} deleted"),
                )
            })
        }
        // Посылки: в 0.10.13 по ним приходят отдельные события, раньше список
        // обновлялся только по кнопке. В контексте контеста перечитываем список
        // и молчим — тост о новой посылке чужого контеста здесь лишний.
        ContestsEvent::NewSubmission(_) | ContestsEvent::SubmissionUpdated(_) => {
            if let Some(cid) = contest_id {
                let pid = STATE.read().selected_problem_id;
                if let Some(pid) = pid {
                    let all = STATE.read().all_submissions;
                    let token = crate::state::token();
                    let res = if all {
                        crate::api::problems::get_problem_submissions_all(pid, &token).await
                    } else {
                        crate::api::problems::get_problem_submissions_my(pid, &token).await
                    };
                    if let Ok(subs) = res {
                        STATE.write().submissions = subs;
                    }
                }
                let _ = cid;
            }
            None
        }
        ContestsEvent::NewProblemQuestion(id) => {
            if let Some(cid) = contest_id {
                reload_questions(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Новый вопрос #{id}"),
                    &format!("New question #{id}"),
                )
            })
        }
        ContestsEvent::ProblemQuestionDeleted(id) => {
            if let Some(cid) = contest_id {
                let n = {
                    let state = STATE.read();
                    state
                        .questions
                        .iter()
                        .position(|q| q.id == *id)
                        .map(|i| (state.questions.len() - i) as i64)
                        .unwrap_or(*id)
                };
                reload_questions(cid).await;
                Some(i18n::tr(
                    &lang,
                    &format!("Вопрос #{n} удалён"),
                    &format!("Question #{n} deleted"),
                ))
            } else {
                None
            }
        }
        ContestsEvent::ProblemQuestionAnswered(id) => {
            if let Some(cid) = contest_id {
                reload_questions(cid).await;
                let state = STATE.read();
                let n = state
                    .questions
                    .iter()
                    .position(|q| q.id == *id)
                    .map(|i| (state.questions.len() - i) as i64)
                    .unwrap_or(*id);
                Some(i18n::tr(
                    &lang,
                    &format!("Ответ на вопрос #{n}"),
                    &format!("Question #{n} answered"),
                ))
            } else {
                None
            }
        }
    };
    if let Some(text) = notice {
        show_alert(AlertKind::Info, text);
    }
}

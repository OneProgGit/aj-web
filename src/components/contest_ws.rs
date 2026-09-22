use dioxus::prelude::*;
use futures_util::{FutureExt, SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use wasm_bindgen_futures::spawn_local;

use crate::{
    alerts::{show_alert, AlertKind},
    i18n,
    models::contests::ContestEvent,
    state::STATE,
};

/// Вставка/замена с сохранением серверного порядка (`c.id desc`):
/// существующий — заменяется на месте (id неизменен, порядок цел),
/// отсутствующий (новый, отжатый "скрыть") — встаёт по своему id.
fn upsert_contest(list: &mut Vec<crate::models::contests::PublicContestConfig>, fresh: crate::models::contests::PublicContestConfig) {
    if let Some(slot) = list.iter_mut().find(|c| c.id == fresh.id) {
        *slot = fresh;
        return;
    }
    let pos = list.iter().position(|c| fresh.id > c.id).unwrap_or(list.len());
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
                crate::state::WS_SUBSCRIBED.write().remove(&id);
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

const WS_RETRY_MS: u32 = 2_000;
/// Heartbeat чаще, чем типичные idle-таймауты NAT/прокси (~25-30с),
/// иначе молчащий сокет режут. Сервер текстовые сообщения игнорирует.
const WS_PING_MS: u32 = 15_000;
const WS_HEARTBEAT: &str = "__ping__";

/// Открывает ws на /contests/{id}/ws?token= (Bearer-токен из localStorage,
/// заголовки браузерный WebSocket слать не умеет) и на каждое событие
/// обновляет глобальное состояние.
pub fn contest_ws(contest_id: i64) {
    if !crate::state::WS_SUBSCRIBED.write().insert(contest_id) {
        return;
    }
    spawn_local(async move {
        loop {
            if !crate::state::WS_SUBSCRIBED.read().contains(&contest_id) {
                break;
            }
            ws_loop(&ws_url(&format!("/contests/{contest_id}/ws")), |event| {
                handle_event(Some(contest_id), event)
            })
            .await;
            gloo_timers::future::TimeoutFuture::new(WS_RETRY_MS).await;
        }
    });
}

/// Подписка списка контестов: все (`/contests/ws`) или свои (`/contests/my/ws`).
/// Обрабатывает только членство списка (создание/обновление/удаление),
/// чтобы не спамить тостами постов чужих контестов.
pub fn contests_feed_ws(mine: bool) {
    const FEED_ALL_ID: i64 = -1;
    const FEED_MY_ID: i64 = -2;
    let key = if mine { FEED_MY_ID } else { FEED_ALL_ID };
    if !crate::state::WS_SUBSCRIBED.write().insert(key) {
        return;
    }
    spawn_local(async move {
        let path = if mine { "/contests/my/ws" } else { "/contests/ws" };
        loop {
            if !crate::state::WS_SUBSCRIBED.read().contains(&key) {
                break;
            }
            ws_loop(&ws_url(path), |event| handle_feed_event(event)).await;
            gloo_timers::future::TimeoutFuture::new(WS_RETRY_MS).await;
        }
    });
}

fn ws_url(path: &str) -> String {
    let base = crate::api::api_base();
    let scheme = if base.starts_with("https") {
        "wss"
    } else {
        "ws"
    };
    let host_port = base
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    let url = format!("{scheme}://{host_port}{path}");
    url
}

/// Один цикл подключения (разрыв снаружи — обычный ретрай).
async fn ws_loop<Fut>(url: &str, on_event: impl Fn(ContestEvent) -> Fut)
where
    Fut: std::future::Future<Output = ()>,
{
    if let Ok(socket) = WebSocket::open(url) {
        let (mut sink, mut stream) = socket.split();
        loop {
            let timer = Box::pin(gloo_timers::future::TimeoutFuture::new(WS_PING_MS).fuse());
            futures_util::select! {
                msg = stream.next().fuse() => {
                    let Some(Ok(Message::Text(text))) = msg else { break };
                    if text == WS_HEARTBEAT {
                        continue;
                    }
                    if let Ok(event) = serde_json::from_str::<ContestEvent>(&text) {
                        on_event(event).await;
                    }
                },
                _ = timer.fuse() => {
                    if sink.send(Message::Text(WS_HEARTBEAT.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}

async fn handle_feed_event(event: ContestEvent) {
    handle_event(None, event).await;
}

async fn handle_event(contest_id: Option<i64>, event: ContestEvent) {
    let lang = crate::state::language();
    // В фиде (контекста контеста нет) — только членство списка.
    let notice = match &event {
        ContestEvent::NewPost(id) => {
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
        ContestEvent::PostUpdated(id) => {
            if let Some(cid) = contest_id {
                reload_posts(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Объявление #{id} обновлено"),
                    &format!("Announcement #{id} updated"),
                )
            })
        }
        ContestEvent::PostDeleted(id) => {
            if let Some(cid) = contest_id {
                reload_posts(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Объявление #{id} удалено"),
                    &format!("Announcement #{id} deleted"),
                )
            })
        }
        ContestEvent::ContestUpdated(id) => {
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
        ContestEvent::ContestDeleted(id) => {
            // Тост только если контест был у нас в списке; чужой/скрытый — тихо.
            let mut state = STATE.write();
            let had = state.contests.iter().any(|c| c.id == *id);
            state.contests.retain(|c| c.id != *id);
            drop(state);
            crate::state::WS_SUBSCRIBED.write().remove(id);
            had.then(|| {
                i18n::tr(
                    &lang,
                    &format!("Контест #{id} удалён"),
                    &format!("Contest #{id} deleted"),
                )
            })
        }
        ContestEvent::NewContest(id) => {
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
        ContestEvent::NewProblem(id) => {
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
        ContestEvent::ProblemUpdated(id) => {
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
        ContestEvent::ProblemDeleted(id) => {
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
        ContestEvent::NewProblemQuestion(id) => {
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
        ContestEvent::ProblemQuestionDeleted(id) => {
            if let Some(cid) = contest_id {
                reload_questions(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Вопрос #{id} удалён"),
                    &format!("Question #{id} deleted"),
                )
            })
        }
        ContestEvent::ProblemQuestionAnswered(id) => {
            if let Some(cid) = contest_id {
                reload_questions(cid).await;
            }
            contest_id.map(|_| {
                i18n::tr(
                    &lang,
                    &format!("Ответ на вопрос #{id}"),
                    &format!("Question #{id} answered"),
                )
            })
        }
    };
    if let Some(text) = notice {
        show_alert(AlertKind::Info, text);
    }
}

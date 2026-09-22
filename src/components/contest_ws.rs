use dioxus::prelude::*;
use futures_util::{FutureExt, SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use wasm_bindgen_futures::spawn_local;

use crate::{
    alerts::{show_alert, AlertKind},
    i18n,
    models::contests::{ContestEvent, PublicContestConfig},
    state::STATE,
};

/// Полный GET контеста с фолбэком на данные события.
async fn refresh_contest(id: i64, contest: PublicContestConfig) {
    let token = crate::state::token();
    match crate::api::contests::get_contest(id, &token).await {
        Ok(fresh) => {
            let mut state = STATE.write();
            if let Some(slot) = state.contests.iter_mut().find(|c| c.id == id) {
                *slot = fresh;
            } else {
                state.contests.push(fresh);
            }
        }
        Err(_) => {
            let mut state = STATE.write();
            if let Some(slot) = state.contests.iter_mut().find(|c| c.id == id) {
                *slot = contest;
            } else {
                state.contests.push(contest);
            }
        }
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
                handle_event(contest_id, event)
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
    let mut url = format!("{scheme}://{host_port}{path}");
    if let Some(token) = crate::state::token() {
        url.push_str(&format!("?token={token}"));
    }
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
    let lang = crate::state::language();
    match event {
        ContestEvent::NewContest(contest) => {
            let id = contest.id;
            let mut state = STATE.write();
            if !state.contests.iter().any(|c| c.id == id) {
                state.contests.insert(0, contest);
                show_alert(
                    AlertKind::Info,
                    i18n::tr(
                        &lang,
                        &format!("Контест #{id} создан"),
                        &format!("Contest #{id} created"),
                    ),
                );
            }
        }
        ContestEvent::ContestUpdated(contest) => {
            refresh_contest(contest.id, contest).await;
        }
        ContestEvent::ContestDeleted(id) => {
            STATE.write().contests.retain(|c| c.id != id);
            show_alert(
                AlertKind::Info,
                i18n::tr(
                    &lang,
                    &format!("Контест #{id} удалён"),
                    &format!("Contest #{id} deleted"),
                ),
            );
        }
        _ => {}
    }
}

async fn handle_event(_contest_id: i64, event: ContestEvent) {
    let lang = crate::state::language();
    let notice = match &event {
        ContestEvent::NewPost(post) => Some(i18n::tr(
            &lang,
            &format!("Новое объявление #{}", post.id),
            &format!("New announcement #{}", post.id),
        )),
        ContestEvent::PostUpdated(post) => Some(i18n::tr(
            &lang,
            &format!("Объявление #{} обновлено", post.id),
            &format!("Announcement #{} updated", post.id),
        )),
        ContestEvent::PostDeleted(idx) => Some(i18n::tr(
            &lang,
            &format!("Объявление #{} удалено", idx),
            &format!("Announcement #{} deleted", idx),
        )),
        ContestEvent::ContestUpdated(contest) => Some(i18n::tr(
            &lang,
            &format!("Контест #{} обновлён", contest.id),
            &format!("Contest #{} updated", contest.id),
        )),
        ContestEvent::ContestDeleted(id) => Some(i18n::tr(
            &lang,
            &format!("Контест #{id} удалён"),
            &format!("Contest #{id} deleted"),
        )),
        ContestEvent::NewContest(contest) => Some(i18n::tr(
            &lang,
            &format!("Контест #{} создан", contest.id),
            &format!("Contest #{} created", contest.id),
        )),
        ContestEvent::NewProblem(problem) => Some(i18n::tr(
            &lang,
            &format!("Новая задача #{}", problem.index + 1),
            &format!("New problem #{}", problem.index + 1),
        )),
        ContestEvent::ProblemUpdated(problem) => Some(i18n::tr(
            &lang,
            &format!("Задача #{} обновлена", problem.index + 1),
            &format!("Problem #{} updated", problem.index + 1),
        )),
        ContestEvent::ProblemDeleted(idx) => Some(i18n::tr(
            &lang,
            &format!("Задача #{} удалена", idx + 1),
            &format!("Problem #{} deleted", idx + 1),
        )),
        ContestEvent::NewProblemQuestion(q) => Some(i18n::tr(
            &lang,
            &format!("Новый вопрос #{}", q.id),
            &format!("New question #{}", q.id),
        )),
        ContestEvent::ProblemQuestionDeleted(idx) => Some(i18n::tr(
            &lang,
            &format!("Вопрос #{} удалён", idx),
            &format!("Question #{} deleted", idx),
        )),
        ContestEvent::ProblemQuestionAnswered(q) => Some(i18n::tr(
            &lang,
            &format!("Ответ на вопрос #{}", q.id),
            &format!("Question #{} answered", q.id),
        )),
    };
    match event {
        ContestEvent::NewPost(post) | ContestEvent::PostUpdated(post) => {
            let mut state = STATE.write();
            if let Some(slot) = state.posts.iter_mut().find(|p| p.id == post.id) {
                *slot = post;
            } else {
                state.posts.insert(0, post);
            }
        }
        ContestEvent::PostDeleted(idx) => {
            STATE.write().posts.retain(|p| p.id != idx);
        }
        ContestEvent::NewProblem(problem) | ContestEvent::ProblemUpdated(problem) => {
            let mut state = STATE.write();
            if let Some(slot) = state
                .contest_problems
                .iter_mut()
                .find(|p| p.id == problem.id)
            {
                *slot = problem;
            } else {
                state.contest_problems.push(problem);
            }
        }
        ContestEvent::ProblemDeleted(idx) => {
            STATE
                .write()
                .contest_problems
                .retain(|p| p.id != idx);
        }
        ContestEvent::NewProblemQuestion(q) | ContestEvent::ProblemQuestionAnswered(q) => {
            let mut state = STATE.write();
            if let Some(slot) = state.questions.iter_mut().find(|x| x.id == q.id) {
                *slot = q;
            } else {
                state.questions.insert(0, q);
            }
        }
        ContestEvent::ProblemQuestionDeleted(idx) => {
            STATE.write().questions.retain(|q| q.id != idx);
        }
        ContestEvent::ContestUpdated(contest) => {
            // Чужое изменение (не из этого интерфейса): забираем свежий
            // контест с сервера целиком, а не доверяем полям события.
            refresh_contest(contest.id, contest).await;
        }
        ContestEvent::ContestDeleted(id) => {
            STATE.write().contests.retain(|c| c.id != id);
            crate::state::WS_SUBSCRIBED.write().remove(&id);
        }
        ContestEvent::NewContest(contest) => {
            let mut state = STATE.write();
            if !state.contests.iter().any(|c| c.id == contest.id) {
                state.contests.insert(0, contest);
            }
        }
    }
    if let Some(text) = notice {
        show_alert(AlertKind::Info, text);
    }
}

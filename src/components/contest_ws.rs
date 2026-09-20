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
            let base = crate::api::api_base();
            let scheme = if base.starts_with("https") { "wss" } else { "ws" };
            let host_port = base
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_end_matches('/');
            let mut url = format!("{scheme}://{host_port}/contests/{contest_id}/ws");
            if let Some(token) = crate::state::token() {
                url.push_str(&format!("?token={token}"));
            }
            if let Ok(socket) = WebSocket::open(&url) {
                    let (mut sink, mut stream) = socket.split();
                    loop {
                        let timer = Box::pin(
                            gloo_timers::future::TimeoutFuture::new(WS_PING_MS).fuse(),
                        );
                        futures_util::select! {
                            msg = stream.next().fuse() => {
                                let Some(Ok(Message::Text(text))) = msg else { break };
                                if text == WS_HEARTBEAT {
                                    continue;
                                }
                                if let Ok(event) = serde_json::from_str::<ContestEvent>(&text) {
                                    handle_event(contest_id, event).await;
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
            gloo_timers::future::TimeoutFuture::new(WS_RETRY_MS).await;
        }
    });
}

async fn handle_event(contest_id: i64, event: ContestEvent) {
    let lang = crate::state::language();
    let notice = match &event {
        ContestEvent::NewPost(post) => Some(i18n::tr(
            &lang,
            &format!("Новое объявление #{}", post.index + 1),
            &format!("New announcement #{}", post.index + 1),
        )),
        ContestEvent::PostUpdated(post) => Some(i18n::tr(
            &lang,
            &format!("Объявление #{} обновлено", post.index + 1),
            &format!("Announcement #{} updated", post.index + 1),
        )),
        ContestEvent::PostDeleted(idx) => Some(i18n::tr(
            &lang,
            &format!("Объявление #{} удалено", idx + 1),
            &format!("Announcement #{} deleted", idx + 1),
        )),
        ContestEvent::ContestUpdated(_) => {
            Some(i18n::tr(&lang, "Контест обновлён", "Contest updated"))
        }
        ContestEvent::ContestDeleted => Some(i18n::tr(&lang, "Контест удалён", "Contest deleted")),
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
            &format!("Новый вопрос #{}", q.index + 1),
            &format!("New question #{}", q.index + 1),
        )),
        ContestEvent::ProblemQuestionDeleted(idx) => Some(i18n::tr(
            &lang,
            &format!("Вопрос #{} удалён", idx + 1),
            &format!("Question #{} deleted", idx + 1),
        )),
        ContestEvent::ProblemQuestionAnswered(q) => Some(i18n::tr(
            &lang,
            &format!("Ответ на вопрос #{}", q.index + 1),
            &format!("Question #{} answered", q.index + 1),
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
            STATE.write().posts.retain(|p| p.index != idx as i64);
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
                .retain(|p| p.index != idx as i64);
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
            STATE.write().questions.retain(|q| q.index != idx as i64);
        }
        ContestEvent::ContestUpdated(contest) => {
            // Чужое изменение (не из этого интерфейса): забираем свежий
            // контест с сервера целиком, а не доверяем полям события.
            let id = contest.id;
            let token = crate::state::token();
            spawn_local(async move {
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
                        // Фолбэк: хотя бы данные из события.
                        let mut state = STATE.write();
                        if let Some(slot) = state.contests.iter_mut().find(|c| c.id == id) {
                            *slot = contest;
                        } else {
                            state.contests.push(contest);
                        }
                    }
                }
            });
        }
        ContestEvent::ContestDeleted => {
            STATE.write().contests.retain(|c| c.id != contest_id);
            crate::state::WS_SUBSCRIBED.write().remove(&contest_id);
        }
    }
    if let Some(text) = notice {
        show_alert(AlertKind::Info, text);
    }
}

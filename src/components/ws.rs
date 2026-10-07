//! Общий транспорт для всех ws-лент: адрес с токеном, heartbeat, ретраи и
//! учёт подписок. Разделено от обработчиков, потому что лент теперь три
//! (контесты, задачи, пользователи), а набор событий у каждого свой.

use std::collections::HashMap;
use std::sync::Mutex;

use dioxus::prelude::ReadableExt;
use futures_util::{FutureExt, SinkExt, StreamExt};
use gloo_net::websocket::Message;
use gloo_net::websocket::futures::WebSocket;
use wasm_bindgen_futures::spawn_local;

use crate::state::WS_SUBSCRIBED;

pub const WS_RETRY_MS: u32 = 2_000;
/// Heartbeat чаще, чем типичные idle-таймауты NAT/прокси (~25-30с),
/// иначе молчащий сокет режут. Сервер текстовые сообщения игнорирует.
pub const WS_PING_MS: u32 = 15_000;
pub const WS_HEARTBEAT: &str = "__ping__";

/// Полный ws-адрес с токеном в query: браузерный WebSocket не умеет
/// заголовки, поэтому Bearer-токен едет параметром.
pub fn ws_url(path: &str) -> String {
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
        url.push_str(if path.contains('?') {
            "&token="
        } else {
            "?token="
        });
        url.push_str(&token);
    }
    url
}

/// Открывает подписку и держит её до отписки (`WS_SUBSCRIBED`).
///
/// `key` — уникальный идентификатор подписки, см. [`WS_SUBSCRIBED`]:
/// повторный вызов с тем же ключом молча ничего не делает, поэтому вызов
/// безопасно делать из каждого рендера. Соединение переподключается само
/// после разрыва, пока подписка жива в множестве.
pub fn subscribe_ws<E, Fut, H>(key: String, path: String, on_event: H)
where
    E: serde::de::DeserializeOwned + 'static,
    Fut: std::future::Future<Output = ()>,
    H: Fn(E) -> Fut + 'static,
{
    if !WS_SUBSCRIBED.write().insert(key.clone()) {
        return;
    }
    spawn_local(async move {
        while WS_SUBSCRIBED.read().contains(key.as_str()) {
            ws_loop(&ws_url(&path), &on_event).await;
            gloo_timers::future::TimeoutFuture::new(WS_RETRY_MS).await;
        }
        // Подписку снимаем только здесь: снять её может и страница, уйдя с
        // экрана, — тогда цикл завершится на следующей проверке.
        WS_SUBSCRIBED.write().remove(&key);
    });
}

/// Закрывает подписку: цикл переподключения увидит отсутствие ключа и выйдет.
pub fn unsubscribe_ws(key: &str) {
    WS_SUBSCRIBED.write().remove(key);
}

/// Метки «только что изменил сам», по лентам.
///
/// Событие, пришедшее следом за собственным изменением, — это его эхо:
/// свой тост обработчик уже показал, а повтор из ленты — дубль. Данные при
/// этом обновляются как обычно, глушится только уведомление. Окно в 3 секунды:
/// чужое событие внутри него тоже замьютится, но состояние всё равно
/// обновится — пострадает только тост, что приемлемо.
// Метки — миллисекунды `js_sys::Date::now()`, а НЕ `std::time::Instant`:
// Instant на wasm32-unknown-unknown не реализован и паникует
// ("time not implemented on this platform"), а непойманная паника в wasm
// убивает инстанс целиком — вкладка виснет. Тот же приём уже используется
// в alerts.rs для дедупликации тостов.
static SELF_MUTE: std::sync::LazyLock<Mutex<HashMap<String, f64>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
const SELF_MUTE_WINDOW_MS: f64 = 3000.0;

/// Отметить собственное изменение в ленте `feed` (`"users"`, `"problems"`).
pub fn mark_self_action(feed: &str) {
    if let Ok(mut map) = SELF_MUTE.lock() {
        map.insert(feed.to_string(), js_sys::Date::now());
    }
}

/// Это эхо собственного недавнего изменения?
#[must_use]
pub fn is_self_echo(feed: &str) -> bool {
    SELF_MUTE
        .lock()
        .ok()
        .and_then(|map| map.get(feed).copied())
        .is_some_and(|t| js_sys::Date::now() - t < SELF_MUTE_WINDOW_MS)
}

/// Закрывает все подписки разом — при выходе и удалении аккаунта, чтобы
/// фоновые сокеты не продолжали ретраиться с мёртвым токеном и не сыпали
/// ошибками авторизации поверх тоста об успешном выходе.
pub fn unsubscribe_all_ws() {
    WS_SUBSCRIBED.write().clear();
}

#[must_use]
pub fn is_subscribed(key: &str) -> bool {
    WS_SUBSCRIBED.read().contains(key)
}

/// Один цикл соединения (разрыв снаружи — обычный ретрай).
pub async fn ws_loop<E, Fut>(url: &str, on_event: &impl Fn(E) -> Fut)
where
    E: serde::de::DeserializeOwned,
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
                    // Неизвестное событие игнорируем: набор вариантов задаёт
                    // сервер, и новое в aj-models не должно ронять ленту.
                    if let Ok(event) = serde_json::from_str::<E>(&text) {
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

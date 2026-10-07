//! Лента пользователей: `/users/ws` для списка и `/users/{id}/ws` для
//! отдельного профиля.
//!
//! По канонам лент: событие молча обновляет данные, без тостов. Список
//! обновляется в `STATE.users`, а одиночная подписка отдаёт событие наверх:
//! страница профиля сама решает, что перечитать, — лента не знает про UI.

use crate::{
    alerts::{AlertKind, show_alert},
    api, i18n,
    models::users::{PrivateUserData, UsersEvent},
    state::STATE,
};

use super::ws::{is_self_echo, subscribe_ws, unsubscribe_ws};

/// Ключ ленты всех пользователей.
pub const USERS_FEED_KEY: &str = "users:all";

/// Ключ подписки на конкретного пользователя.
#[must_use]
pub fn user_key(user_id: i64) -> String {
    format!("user:{user_id}")
}

fn upsert_user(list: &mut Vec<PrivateUserData>, fresh: PrivateUserData) {
    if let Some(slot) = list.iter_mut().find(|u| u.id == fresh.id) {
        *slot = fresh;
        return;
    }
    list.push(fresh);
}

/// Перечитывает пользователя в общий список. Возвращает `false`, если он нам
/// недоступен: тогда тихо убираем из списка, как `refresh_contest` делает
/// для скрытого контеста.
async fn refresh_user_in_list(id: i64) -> bool {
    let token = crate::state::token();
    match api::users::get_private_user(id, &token).await {
        Ok(fresh) => {
            upsert_user(&mut STATE.write().users, fresh);
            true
        }
        Err(_) => {
            STATE.write().users.retain(|u| u.id != id);
            false
        }
    }
}

async fn handle_feed_event(event: UsersEvent) {
    // Уведомления — синие (Info): событие чужое, зелёный «успех» здесь
    // неуместен — мы ничего не делали.
    let lang = crate::state::language();
    let notice = match event {
        UsersEvent::NewUser(id) => {
            refresh_user_in_list(id).await;
            Some(i18n::tr(
                &lang,
                &format!("Новый пользователь #{id}"),
                &format!("New user #{id}"),
            ))
        }
        UsersEvent::UserUpdated(id) => {
            refresh_user_in_list(id).await;
            Some(i18n::tr(
                &lang,
                &format!("Пользователь #{id} обновлён"),
                &format!("User #{id} updated"),
            ))
        }
        UsersEvent::UserDeleted(id) => {
            STATE.write().users.retain(|u| u.id != id);
            Some(i18n::tr(
                &lang,
                &format!("Пользователь #{id} удалён"),
                &format!("User #{id} deleted"),
            ))
        }
    };
    // Эхо собственного изменения (удалил/обновил сам) — тост уже показан
    // в обработчике, повтор из ленты не нужен. Данные выше уже обновлены.
    if let Some(text) = notice
        && !is_self_echo("users")
    {
        show_alert(AlertKind::Info, text);
    }
}

/// Лента всех пользователей — админский список на `/users`.
pub fn users_feed_ws() {
    subscribe_ws(
        USERS_FEED_KEY.to_string(),
        "/users/ws".to_string(),
        handle_feed_event,
    );
}

/// Подписка на одного пользователя: `/users/{id}/ws`.
///
/// Событие о нём обновляет общий список и зовёт `on_update` — страница сама
/// решает, что перечитать в свой сигнал, потому что типы данных у экранов
/// разные (`PublicUserData` у публичного профиля, `PrivateUserData` у
/// приватного). Безопасно вызывать из рендера.
pub fn user_ws<F, Fut>(user_id: i64, on_update: F)
where
    F: Fn(UsersEvent) -> Fut + 'static,
    Fut: std::future::Future<Output = ()> + 'static,
{
    // Rc: замыкание Fn вызывается много раз, а future из `async move` забирает
    // его целиком — без Rc компилятор не даёт сделать и то и другое.
    let on_update = std::rc::Rc::new(on_update);
    subscribe_ws(user_key(user_id), format!("/users/{user_id}/ws"), {
        move |event| {
            let on_update = on_update.clone();
            async move {
                if let UsersEvent::UserUpdated(id) | UsersEvent::NewUser(id) = event
                    && id == user_id
                {
                    let token = crate::state::token();
                    if let Ok(fresh) = api::users::get_private_user(id, &token).await {
                        upsert_user(&mut STATE.write().users, fresh);
                    }
                } else if let UsersEvent::UserDeleted(id) = event
                    && id == user_id
                {
                    STATE.write().users.retain(|u| u.id != id);
                }
                on_update(event).await;
            }
        }
    });
}

/// Закрывает одиночную подписку — страница вызывает при уходе с экрана.
pub fn unsubscribe_user_ws(user_id: i64) {
    unsubscribe_ws(&user_key(user_id));
}

/// Подписка на свой профиль: `/users/me/ws`.
///
/// Удобнее `user_ws(my_id)`: id знать не нужно (сервер берёт его из токена),
/// а сама подписка уже за слоем авторизации — в отличие от публичного
/// `/users/{id}/ws`.
pub fn own_profile_ws<F, Fut>(on_update: F)
where
    F: Fn(UsersEvent) -> Fut + 'static,
    Fut: std::future::Future<Output = ()> + 'static,
{
    subscribe_ws("user:me".to_string(), "/users/me/ws".to_string(), on_update);
}

pub fn unsubscribe_own_profile_ws() {
    unsubscribe_ws("user:me");
}

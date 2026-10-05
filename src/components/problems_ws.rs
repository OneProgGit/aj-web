//! Лента задач: `/problems/ws` (все) и `/problems/my/ws` (свои).
//!
//! По канонам ленты контестов: событие молча обновляет список, без тостов —
//! иначе чужая правка задачи засветилась бы каждому. Исключение: скрытая
//! задача, до которой у нас нет доступа, молча исчезает, как и у контеста.

use crate::{
    api,
    models::problems::{ProblemsEvent, PublicProblemConfig},
    state::STATE,
};

use super::ws::subscribe_ws;

/// Ключ подписки на ленту задач.
#[must_use]
pub fn problems_feed_key(mine: bool) -> String {
    if mine {
        "problems:my".to_string()
    } else {
        "problems:all".to_string()
    }
}

/// Полный GET задачи. Возвращает `false`, если задача нам недоступна.
async fn refresh_problem(id: i64) -> bool {
    let token = crate::state::token();
    match api::problems::get_problem_by_id_admin(id, &token).await {
        Ok(fresh) => {
            upsert_problem(&mut STATE.write().problems, fresh);
            true
        }
        Err(_) => false,
    }
}

/// Вставка/замена с сохранением серверного порядка (`id desc`) — так же,
/// как `upsert_contest`.
fn upsert_problem(list: &mut Vec<PublicProblemConfig>, fresh: PublicProblemConfig) {
    if let Some(slot) = list.iter_mut().find(|p| p.id == fresh.id) {
        *slot = fresh;
        return;
    }
    let pos = list
        .iter()
        .position(|p| fresh.id > p.id)
        .unwrap_or(list.len());
    list.insert(pos, fresh);
}

async fn handle_feed_event(event: ProblemsEvent) {
    match event {
        ProblemsEvent::NewProblem(id) | ProblemsEvent::ProblemUpdated(id) => {
            // В ленте «свои» чужая задача просто не вернётся — и в списке её
            // не будет, ровно как до события.
            refresh_problem(id).await;
        }
        ProblemsEvent::ProblemDeleted(id) => {
            STATE.write().problems.retain(|p| p.id != id);
        }
    }
}

/// Открывает ленту задач. Безопасно вызывать из рендера: повторный вызов
/// с тем же ключом ничего не делает.
pub fn problems_feed_ws(mine: bool) {
    let path = if mine {
        "/problems/my/ws"
    } else {
        "/problems/ws"
    };
    subscribe_ws(problems_feed_key(mine), path.to_string(), handle_feed_event);
}

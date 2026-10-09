//! Лента задач: `/problems/ws` (общая; отдельного `/problems/my/ws` нет).
//!
//! По канонам ленты контестов: событие молча обновляет список, без тостов —
//! иначе чужая правка задачи засветилась бы каждому. Исключение: скрытая
//! задача, до которой у нас нет доступа, молча исчезает, как и у контеста.

use crate::{
    alerts::{AlertKind, show_alert},
    api, i18n,
    models::problems::{ProblemsEvent, PublicProblemConfig},
    state::STATE,
};

use super::ws::{is_self_echo, subscribe_ws};

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
    // Уведомления — синие (Info): лента чужая, событие может прийти от
    // кого угодно, и зелёный «успех» здесь неуместен — мы ничего не делали.
    let lang = crate::state::language();
    let notice = match event {
        ProblemsEvent::NewProblem(id) => {
            // В ленте «свои» чужая задача просто не вернётся — и в списке её
            // не будет, ровно как до события.
            refresh_problem(id).await;
            Some(i18n::tr(
                &lang,
                &format!("Новая задача #{id}"),
                &format!("New problem #{id}"),
            ))
        }
        ProblemsEvent::ProblemUpdated(id) => {
            refresh_problem(id).await;
            Some(i18n::tr(
                &lang,
                &format!("Задача #{id} обновлена"),
                &format!("Problem #{id} updated"),
            ))
        }
        ProblemsEvent::ProblemDeleted(id) => {
            STATE.write().problems.retain(|p| p.id != id);
            Some(i18n::tr(
                &lang,
                &format!("Задача #{id} удалена"),
                &format!("Problem #{id} deleted"),
            ))
        }
    };
    if let Some(text) = notice
        && !is_self_echo("problems")
    {
        show_alert(AlertKind::Info, text);
    }
}

/// Открывает ленту задач. Безопасно вызывать из рендера: повторный вызов
/// с тем же ключом ничего не делает.
///
/// Внимание: отдельного `/problems/my/ws` на бэкенде НЕТ (проверено по
/// роутам ada-judge — есть только `/problems/ws`), поэтому оба режима
/// подписаны на общую ленту. Подписка на несуществующий путь давала
/// бесконечный цикл `WebSocket connection failed` + ретрай каждые 2 секунды.
/// Параметр `mine` оставлен для совместимости: если роут появится, вернуть
/// выбор пути — одна строка. Тумблер при этом всё равно owner-only, а
/// `refresh_problem` недоступное молча игнорирует.
pub fn problems_feed_ws() {
    subscribe_ws(
        "problems:all".to_string(),
        "/problems/ws".to_string(),
        handle_feed_event,
    );
}

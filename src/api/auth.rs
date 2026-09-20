use crate::{
    api,
    models::users::{LoginRequest, RegisterRequest},
};

/// POST /login → JSON string JWT
pub async fn login(login: &str, password: &str) -> Result<String, String> {
    let body = LoginRequest {
        login: login.to_string(),
        password: password.to_string(),
    };
    let res = api::client(&None)
        .post(format!("{}/login", api::api_base()))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Сетевая ошибка", "Network error")))?;
    if res.status().is_success() {
        let token = res
            .text()
            .await
            .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка ответа", "Response error")))?;
        Ok(token.trim_matches('"').to_string())
    } else {
        let error: crate::models::errors::AdaJudgeError = res
            .json()
            .await
            .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка сервера", "Server error")))?;
        Err(crate::models::errors::describe_error(&error))
    }
}

/// POST /login/cookie → 200 on success.
/// Same credentials; server sets the HttpOnly Secure `token` cookie,
/// which the browser then sends automatically on the ws handshake.
#[allow(dead_code)]
pub async fn login_cookie(login: &str, password: &str) -> Result<(), String> {
    let body = LoginRequest {
        login: login.to_string(),
        password: password.to_string(),
    };
    let res = api::client(&None)
        .post(format!("{}/login/cookie", api::api_base()))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Сетевая ошибка", "Network error")))?;
    if res.status().is_success() {
        Ok(())
    } else {
        let error: crate::models::errors::AdaJudgeError = res
            .json()
            .await
            .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка сервера", "Server error")))?;
        Err(crate::models::errors::describe_error(&error))
    }
}

/// POST /register → 200 on success
pub async fn register(
    login: &str,
    password: &str,
    password_confirmation: &str,
) -> Result<(), String> {
    let body = RegisterRequest {
        login: login.to_string(),
        password: password.to_string(),
        password_confirmation: password_confirmation.to_string(),
    };
    let res = api::client(&None)
        .post(format!("{}/register", api::api_base()))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Сетевая ошибка", "Network error")))?;
    if res.status().is_success() {
        Ok(())
    } else {
        let error: crate::models::errors::AdaJudgeError = res
            .json()
            .await
            .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка сервера", "Server error")))?;
        Err(crate::models::errors::describe_error(&error))
    }
}

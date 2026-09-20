use crate::models::errors::AdaJudgeError;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Backend base URL. Defaults to the production host used by aj-app;
/// override at build time with `API_BASE` (still baked into wasm).
pub fn api_base() -> &'static str {
    option_env!("API_BASE").unwrap_or("https://aj-host.oneprog.org")
}

pub mod auth;
pub mod contests;
pub mod problems;
pub mod users;

#[allow(dead_code)]
pub fn client(_token: &Option<String>) -> reqwest::Client {
    reqwest::Client::new()
}

pub fn auth_header(token: &Option<String>) -> Option<String> {
    token.as_ref().map(|token| format!("Bearer {token}"))
}

fn endpoint(path: &str) -> String {
    format!("{}{path}", api_base())
}

/// Send the request builder with the token; on non-2xx parse the backend
/// `AdaJudgeError` into a localized Russian message.
async fn check(res: reqwest::Result<reqwest::Response>) -> Result<reqwest::Response, String> {
    let res = res.map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Сетевая ошибка", "Network error")))?;
    if res.status().is_success() {
        Ok(res)
    } else {
        let status = res.status();
        match res.json::<AdaJudgeError>().await {
            Ok(error) => {
                if matches!(error, AdaJudgeError::InvalidJwt) {
                    // Session expired / token revoked by the backend: log the
                    // user out so every guarded route bounces back to Welcome.
                    crate::state::STATE.write().token = None;
                    crate::state::STATE.write().user = None;
                    crate::state::clear_token();
                }
                Err(crate::models::errors::describe_error(&error))
            }
            Err(_) => Err(format!("{}: {status}", crate::i18n::tr(&crate::state::language(), "Ошибка сервера", "Server error"))),
        }
    }
}

pub async fn get<T: serde::de::DeserializeOwned>(
    path: &str,
    token: &Option<String>,
) -> Result<T, String> {
    let mut req = client(token).get(endpoint(path));
    if let Some(auth) = auth_header(token) {
        req = req.header("Authorization", auth);
    }
    let res = check(req.send().await).await?;
    res.json().await.map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка разбора ответа", "Response parse error")))
}

pub async fn get_bytes(path: &str, token: &Option<String>) -> Result<Vec<u8>, String> {
    let mut req = client(token).get(endpoint(path));
    if let Some(auth) = auth_header(token) {
        req = req.header("Authorization", auth);
    }
    let res = check(req.send().await).await?;
    res.bytes().await.map(|b| b.to_vec()).map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка чтения ответа", "Response read error")))
}

pub async fn send_json<B: serde::Serialize>(
    method: reqwest::Method,
    path: &str,
    token: &Option<String>,
    body: &B,
) -> Result<(), String> {
    let mut req = client(token)
        .request(method, endpoint(path))
        .json(body);
    if let Some(auth) = auth_header(token) {
        req = req.header("Authorization", auth);
    }
    let res = check(req.send().await).await?;
    drop(res);
    Ok(())
}

/// Save a byte blob as a browser download (`<a download>` + object URL).
pub fn trigger_download(bytes: Vec<u8>, filename: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let array = js_sys::Array::new();
    array.push(&js_sys::Uint8Array::from(&bytes[..]));
    let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(&array) else {
        return;
    };
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
    let Ok(anchor) = document.create_element("a") else {
        return;
    };
    let anchor = anchor.unchecked_into::<web_sys::HtmlAnchorElement>();
    let _ = anchor.set_attribute("href", &url);
    let _ = anchor.set_attribute("download", filename);
    anchor.click();
    let _ = web_sys::Url::revoke_object_url(&url);
}

pub async fn download(path: &str, token: &Option<String>, filename: &str) -> Result<(), String> {
    let bytes = get_bytes(path, token).await?;
    trigger_download(bytes, filename);
    Ok(())
}

/// Read the full content of a `<input type="file">` picked by the user.
#[allow(dead_code)]
pub async fn read_file_bytes(
    element: &web_sys::HtmlInputElement,
) -> Result<(String, Vec<u8>), String> {
    let file = element
        .files()
        .and_then(|files| files.item(0))
        .ok_or_else(|| "Файл не выбран".to_string())?;
    let name = file.name();
    let buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| crate::i18n::tr(&crate::state::language(), "Не удалось прочитать файл", "Could not read file"))?;
    let typed = js_sys::Uint8Array::new(&buffer);
    Ok((name, typed.to_vec()))
}
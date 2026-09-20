use crate::{
    api,
    models::{
        testing::{Submission, SubmissonRequest},
    },
};
use reqwest::{Method, StatusCode};

pub async fn get_contest_problems(
    contest_id: i64,
    token: &Option<String>,
) -> Result<Vec<crate::models::problems::PublicProblemConfig>, String> {
    api::get(&format!("/contests/{contest_id}/problems"), token).await
}

#[allow(dead_code)]
pub async fn get_problem_by_id_admin(
    problem_id: i64,
    token: &Option<String>,
) -> Result<crate::models::problems::PublicProblemConfig, String> {
    api::get(&format!("/problems/{problem_id}"), token).await
}

pub async fn get_my_problems(
    token: &Option<String>,
) -> Result<Vec<crate::models::problems::PublicProblemConfig>, String> {
    api::get("/problems/my", token).await
}

pub async fn get_all_problems(
    token: &Option<String>,
) -> Result<Vec<crate::models::problems::PublicProblemConfig>, String> {
    api::get("/problems", token).await
}

pub async fn create_problem(
    archive_bytes: Vec<u8>,
    token: &Option<String>,
) -> Result<(), String> {
    let form = reqwest::multipart::Form::new().part(
        "problem_archive",
        reqwest::multipart::Part::bytes(archive_bytes),
    );
    let mut req = api::client(token)
        .post(format!("{}/problems/new", api::api_base()))
        .multipart(form);
    if let Some(auth) = api::auth_header(token) {
        req = req.header("Authorization", auth);
    }
    let res = req
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

pub async fn update_problem(
    problem_id: i64,
    archive_bytes: Vec<u8>,
    token: &Option<String>,
) -> Result<(), String> {
    let form = reqwest::multipart::Form::new().part(
        "problem_archive",
        reqwest::multipart::Part::bytes(archive_bytes),
    );
    let mut req = api::client(token)
        .patch(format!("{}/problems/{problem_id}/update", api::api_base()))
        .multipart(form);
    if let Some(auth) = api::auth_header(token) {
        req = req.header("Authorization", auth);
    }
    let res = req
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

pub async fn delete_problem(
    problem_id: i64,
    deletion: &crate::models::DeletionRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::DELETE,
        &format!("/problems/{problem_id}/delete"),
        token,
        deletion,
    )
    .await
}

pub async fn download_problem(problem_id: i64, token: &Option<String>) -> Result<(), String> {
    api::download(
        &format!("/problems/{problem_id}/download"),
        token,
        &format!("{problem_id}.zip"),
    )
    .await
}

pub async fn retest_problem(problem_id: i64, token: &Option<String>) -> Result<(), String> {
    let mut req = api::client(token).post(format!("{}/problems/{problem_id}/retest", api::api_base()));
    if let Some(auth) = api::auth_header(token) {
        req = req.header("Authorization", auth);
    }
    let res = req
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

pub async fn get_problem_submissions_all(
    problem_id: i64,
    token: &Option<String>,
) -> Result<Vec<Submission>, String> {
    api::get(&format!("/problems/{problem_id}/submissions"), token).await
}

pub async fn get_problem_submissions_my(
    problem_id: i64,
    token: &Option<String>,
) -> Result<Vec<Submission>, String> {
    api::get(&format!("/problems/{problem_id}/submissions/my"), token).await
}

pub async fn download_submission_file(
    submission_id: i64,
    token: &Option<String>,
    filename: &str,
) -> Result<(), String> {
    api::download(
        &format!("/submissions/{submission_id}/download"),
        token,
        filename,
    )
    .await
}

/// POST /contests/{contest_id}/problems/{problem_id}/submit (multipart)
pub async fn submit_solution(
    contest_id: i64,
    problem_id: i64,
    language: crate::models::testing::Language,
    file_bytes: Vec<u8>,
    token: &Option<String>,
) -> Result<i64, String> {
    let json = serde_json::to_string(&SubmissonRequest { language })
        .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка сериализации", "Serialization error")))?;
    let form = reqwest::multipart::Form::new()
        .part(
            "submission_data",
            reqwest::multipart::Part::text(json)
                .mime_str("application/json")
                .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка", "Error")))?,
        )
        .part("submission_file", reqwest::multipart::Part::bytes(file_bytes));
    let mut req = api::client(token).post(format!(
        "{}/contests/{contest_id}/problems/{problem_id}/submit",
        api::api_base()
    ));
    if let Some(auth) = api::auth_header(token) {
        req = req.header("Authorization", auth);
    }
    let res = req
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Сетевая ошибка", "Network error")))?;
    match res.status() {
        StatusCode::OK => res
            .json()
            .await
            .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка ответа", "Response error"))),
        _ => {
            let error: crate::models::errors::AdaJudgeError = res
                .json()
                .await
                .map_err(|e| format!("{}: {e}", crate::i18n::tr(&crate::state::language(), "Ошибка сервера", "Server error")))?;
            Err(crate::models::errors::describe_error(&error))
        }
    }
}
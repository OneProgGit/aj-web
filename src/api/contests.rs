use crate::{
    api,
    models::{
        contests::{
            ContestPost, ContestPostRequest, ContestRequest, LeaderboardRow, PublicContestConfig,
        },
        problems::{ProblemQuestion, ProblemQuestionRequest},
    },
};
use reqwest::Method;

pub async fn get_contests(token: &Option<String>) -> Result<Vec<PublicContestConfig>, String> {
    api::get("/contests", token).await
}

pub async fn get_contest(
    contest_id: i64,
    token: &Option<String>,
) -> Result<PublicContestConfig, String> {
    api::get(&format!("/contests/{contest_id}"), token).await
}

pub async fn get_my_contests(token: &Option<String>) -> Result<Vec<PublicContestConfig>, String> {
    api::get("/contests/my", token).await
}

pub async fn create_contest(
    request: &ContestRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(Method::POST, "/contests/new", token, request).await
}

pub async fn update_contest(
    contest_id: i64,
    request: &ContestRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::PATCH,
        &format!("/contests/{contest_id}/update"),
        token,
        request,
    )
    .await
}

pub async fn delete_contest(
    contest_id: i64,
    deletion: &crate::models::DeletionRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::DELETE,
        &format!("/contests/{contest_id}/delete"),
        token,
        deletion,
    )
    .await
}

pub async fn get_contest_posts(
    contest_id: i64,
    token: &Option<String>,
) -> Result<Vec<ContestPost>, String> {
    api::get(&format!("/contests/{contest_id}/posts"), token).await
}

pub async fn create_contest_post(
    contest_id: i64,
    request: &ContestPostRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::POST,
        &format!("/contests/{contest_id}/posts/new"),
        token,
        request,
    )
    .await
}

pub async fn update_contest_post(
    post_id: i64,
    request: &ContestPostRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::PATCH,
        &format!("/contests/posts/{post_id}/update"),
        token,
        request,
    )
    .await
}

pub async fn delete_contest_post(
    post_id: i64,
    deletion: &crate::models::DeletionRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::DELETE,
        &format!("/contests/posts/{post_id}/delete"),
        token,
        deletion,
    )
    .await
}

pub async fn get_contest_leaderboard(
    contest_id: i64,
    token: &Option<String>,
) -> Result<Vec<LeaderboardRow>, String> {
    api::get(&format!("/contests/{contest_id}/leaderboard"), token).await
}

pub async fn get_contest_questions_all(
    contest_id: i64,
    token: &Option<String>,
) -> Result<Vec<ProblemQuestion>, String> {
    api::get(&format!("/contests/{contest_id}/questions"), token).await
}

pub async fn get_contest_questions_my(
    contest_id: i64,
    token: &Option<String>,
) -> Result<Vec<ProblemQuestion>, String> {
    api::get(&format!("/contests/{contest_id}/questions/my"), token).await
}

pub async fn create_problem_question(
    problem_id: i64,
    request: &ProblemQuestionRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::POST,
        &format!("/problems/{problem_id}/questions/new"),
        token,
        request,
    )
    .await
}

pub async fn answer_problem_question(
    question_id: i64,
    answer: &str,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::PATCH,
        &format!("/problems/questions/{question_id}/answer"),
        token,
        &answer.to_string(),
    )
    .await
}

pub async fn delete_problem_question(
    question_id: i64,
    deletion: &crate::models::DeletionRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::DELETE,
        &format!("/problems/questions/{question_id}/delete"),
        token,
        deletion,
    )
    .await
}
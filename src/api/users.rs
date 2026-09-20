use crate::{
    api,
    models::users::{AdminLevel, PrivateUserData, PublicUserData},
};
use reqwest::Method;

pub async fn get_me(token: &Option<String>) -> Result<PrivateUserData, String> {
    api::get("/users/me", token).await
}

pub async fn get_public_user(
    user_id: i64,
    token: &Option<String>,
) -> Result<PublicUserData, String> {
    api::get(&format!("/users/{user_id}"), token).await
}

pub async fn get_private_user(
    user_id: i64,
    token: &Option<String>,
) -> Result<PrivateUserData, String> {
    api::get(&format!("/users/{user_id}/private"), token).await
}

pub async fn get_all_users(token: &Option<String>) -> Result<Vec<PrivateUserData>, String> {
    api::get("/users", token).await
}

pub async fn update_admin_level(
    user_id: i64,
    admin_level: &AdminLevel,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::PATCH,
        &format!("/users/{user_id}/update_admin_level"),
        token,
        admin_level,
    )
    .await
}

pub async fn delete_my_account(
    deletion: &crate::models::DeletionRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(Method::DELETE, "/users/me/delete_account", token, deletion).await
}

pub async fn delete_user_account(
    user_id: i64,
    deletion: &crate::models::DeletionRequest,
    token: &Option<String>,
) -> Result<(), String> {
    api::send_json(
        Method::DELETE,
        &format!("/users/{user_id}/delete_account"),
        token,
        deletion,
    )
    .await
}
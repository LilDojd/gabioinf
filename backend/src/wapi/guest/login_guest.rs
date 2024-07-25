use crate::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginUserInput {
    pub user: LoginUserInputUserKey,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserInputUserKey {
    pub email: String,
    pub password: String,
}

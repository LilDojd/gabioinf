use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{errors::BResult, AppState};

pub async fn get_all_entries(State(state): State<AppState>) -> BResult<impl IntoResponse> {
    let entries = state.guestbook_crud.get_all_entries().await?;
    Ok((StatusCode::OK, Json(entries)))
}

pub async fn get_naughty_entries(State(state): State<AppState>) -> BResult<impl IntoResponse> {
    let entries = state.guestbook_crud.get_naughty_entries().await?;
    Ok((StatusCode::OK, Json(entries)))
}

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{domain::models::Guest, errors::BResult, AppState};

#[derive(Deserialize, Debug)]
pub struct CreateEntryRequest {
    message: String,
}

pub async fn create_entry(
    State(state): State<AppState>,
    guest: Guest,
    Json(payload): Json<CreateEntryRequest>,
) -> BResult<impl IntoResponse> {
    let entry = state
        .guestbook_crud
        .create_entry(&guest.id, &payload.message)
        .await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UpdateEntryRequest {
    message: String,
}

#[derive(Deserialize)]
pub struct FlagNaughtyRequest {
    reason: String,
}

use crate::{
    domain::models::Guest,
    errors::{BResult, BackendError},
    AppState,
};

pub async fn update_entry(
    State(state): State<AppState>,
    guest: Guest,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateEntryRequest>,
) -> BResult<impl IntoResponse> {
    // TODO: Check if the entry belongs to the guest or if the guest is an admin
    let entry = state
        .guestbook_crud
        .update_entry(id, &payload.message)
        .await?;
    Ok((StatusCode::OK, Json(entry)))
}

pub async fn flag_as_naughty(
    State(state): State<AppState>,
    guest: Guest,
    Path(id): Path<i64>,
    Json(payload): Json<FlagNaughtyRequest>,
) -> BResult<impl IntoResponse> {
    if !guest.is_admin {
        return Err(BackendError::Forbidden(
            "Only admins can flag entries as naughty".to_string(),
        ));
    }
    let entry = state
        .guestbook_crud
        .flag_as_naughty(id, &payload.reason)
        .await?;
    Ok((StatusCode::OK, Json(entry)))
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{domain::models::Guest, errors::BResult, AppState};

pub async fn promote_to_admin(
    State(state): State<AppState>,
    admin: Guest,
    Path(guest_id): Path<i64>,
) -> BResult<impl IntoResponse> {
    if !admin.is_admin {
        return Ok((StatusCode::FORBIDDEN, "Only admins can promote users").into_response());
    }

    let promoted_guest = state.guest_crud.promote_to_admin(guest_id).await?;
    Ok((StatusCode::OK, Json(promoted_guest)).into_response())
}

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{domain::models::Guest, errors::BResult, AppState};

pub async fn get_all_admins(
    State(state): State<AppState>,
    admin: Guest,
) -> BResult<impl IntoResponse> {
    if !admin.is_admin {
        return Ok((StatusCode::FORBIDDEN, "Only admins can view all admins").into_response());
    }

    let admins = state.guest_crud.get_all_admins().await?;
    Ok((StatusCode::OK, Json(admins)).into_response())
}

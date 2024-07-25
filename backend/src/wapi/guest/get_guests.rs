use axum::{extract::State, response::IntoResponse, Json};

use crate::{domain::models::Guest, errors::BResult, AppState};

// TODO: Refactor
pub async fn get_guests(
    State(state): State<AppState>,
) -> Result<Json<Vec<Guest>>, impl IntoResponse> {
    let guests = state.guest_crud.get_guests().await;
    match guests {
        Ok(guests) => Ok(Json(guests)),
        Err(err) => Err(err),
    }
}

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{domain::models::Guest, errors::BResult, AppState};

pub async fn delete_entry(
    State(state): State<AppState>,
    guest: Guest,
    Path(id): Path<i64>,
) -> BResult<impl IntoResponse> {
    // TODO: Check if the entry belongs to the guest or if the guest is an admin
    state.guestbook_crud.delete_entry(id).await?;
    Ok(())
}

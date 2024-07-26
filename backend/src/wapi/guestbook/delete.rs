use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{
    domain::models::Guest,
    errors::{BResult, BackendError},
    AppState,
};

pub async fn delete_entry(
    State(state): State<AppState>,
    guest: Guest,
    Path(id): Path<i64>,
) -> BResult<impl IntoResponse> {
    let entry = state.guestbook_crud.get_entry(id).await?;

    if entry.author_id == guest.id || guest.is_admin {
        state.guestbook_crud.delete_entry(id).await?;
        Ok(())
    } else {
        Err(BackendError::Forbidden(
            "You are not allowed to delete this entry".to_string(),
        ))
    }
}

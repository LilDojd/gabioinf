//! Guestbook entry deletion handler.
//!
//! This module contains the handler function for deleting a guestbook entry,
//! including authorization checks to ensure only the author or an admin can delete an entry.

use crate::{
    domain::models::Guest,
    errors::{ApiError, BResult},
    AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

/// Handler for deleting a guestbook entry.
///
/// This function attempts to delete a guestbook entry identified by its ID.
/// It includes authorization checks to ensure that only the author of the entry
/// or an admin can delete it.
///
/// # Arguments
///
/// * `state` - The application state, containing the guestbook CRUD operations.
/// * `guest` - The authenticated guest making the request.
/// * `id` - The ID of the guestbook entry to delete.
///
/// # Returns
///
/// Returns a `BResult` containing an empty response if successful, or an error if:
/// - The entry doesn't exist
/// - The authenticated user is not authorized to delete the entry
/// - The deletion operation fails
///
/// # Authorization
///
/// This function checks if the authenticated guest is either:
/// - The author of the entry
/// - An admin
///
/// If neither condition is met, an `AuthorizationError` is returned.
///
/// # Response
///
/// * `204 No Content` if the deletion is successful.
/// * `403 Forbidden` if the user is not authorized to delete the entry.
/// * `404 Not Found` if the entry does not exist.
///
/// # Errors
///
/// This function will return an error if:
/// - The entry lookup fails
/// - The user is not authorized to delete the entry
/// - The deletion operation fails
pub async fn delete_entry(
    State(state): State<AppState>,
    guest: Guest,
    Path(id): Path<i64>,
) -> BResult<impl IntoResponse> {
    tracing::debug!("Deleting guestbook entry with ID: {}", id);

    let entry = state.guestbook_crud.get_entry(id).await?;

    if entry.author_id == guest.id || guest.is_admin {
        state.guestbook_crud.delete_entry(id).await?;
        Ok(())
    } else {
        Err(ApiError::AuthorizationError(
            "You are not allowed to delete this entry".to_string(),
        ))
    }
}

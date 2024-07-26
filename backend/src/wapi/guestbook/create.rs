//! Guestbook entry creation handler.
//!
//! This module contains the handler function for creating a new guestbook entry,
//! along with the necessary request payload structure.

use crate::{domain::models::Guest, errors::BResult, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

/// Request payload for creating a new guestbook entry.
#[derive(Deserialize, Debug)]
pub struct CreateEntryRequest {
    /// The message content of the new guestbook entry.
    message: String,
}

/// Handler for creating a new guestbook entry.
///
/// This function creates a new guestbook entry with the provided message,
/// associating it with the authenticated guest.
///
/// # Arguments
///
/// * `state` - The application state, containing the guestbook CRUD operations.
/// * `guest` - The authenticated guest creating the entry.
/// * `payload` - The JSON payload containing the message for the new entry.
///
/// # Returns
///
/// Returns a `BResult` containing the created entry if successful, or an error if the creation fails.
///
/// # Response
///
/// * `201 Created` with a JSON representation of the created guestbook entry.
///
/// # Errors
///
/// This function will return an error if:
/// - The entry creation fails in the database.
/// - Any other unexpected error occurs during the process.
///
/// # Example
///
/// ```json
/// POST /guestbook
/// Content-Type: application/json
///
/// {
///     "message": "Hello, this is my guestbook entry!"
/// }
/// ```
///
/// Successful response:
/// ```json
/// HTTP/1.1 201 Created
/// Content-Type: application/json
///
/// {
///     "id": 1,
///     "message": "Hello, this is my guestbook entry!",
///     "author_id": 123,
///     "created_at": "2023-07-26T12:34:56Z"
/// }
/// ```
pub async fn create_entry(
    State(state): State<AppState>,
    guest: Guest,
    Json(payload): Json<CreateEntryRequest>,
) -> BResult<impl IntoResponse> {
    tracing::debug!("Creating new guestbook entry");
    let entry = state
        .guestbook_crud
        .create_entry(&guest.id, &payload.message)
        .await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

//! Guestbook entry creation handler.
//!
//! This module contains the handler function for creating a new guestbook entry,
//! along with the necessary request payload structure.

use crate::{
    domain::models::{Guest, NewGuestbookEntry},
    errors::BResult,
    repos::Repository,
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use validator::Validate;

/// Request payload for creating a new guestbook entry.
#[derive(Deserialize, Debug, Validate)]
pub struct CreateEntryRequest {
    /// The message content of the new guestbook entry.
    #[validate(
        length(
            min = 1,
            max = 255,
            message = "Message must be between 1 and 255 characters"
        ),
        custom(function = "crate::utils::validate_not_offensive")
    )]
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

    let new_entry = NewGuestbookEntry {
        author_id: guest.id,
        message: payload.message.trim().to_string(),
        // TODO: Change this
        signature: None,
    }
    .into();

    let entry = state.guestbook_repo.create(&new_entry).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

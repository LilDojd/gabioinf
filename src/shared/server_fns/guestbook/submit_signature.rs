//! Guestbook entry creation handler.
//!
//! This module contains the handler function for creating a new guestbook entry,
//! along with the necessary request payload structure.
#[cfg(feature = "server")]
use crate::backend::{AppState, domain::logic::SessionWrapper, errors::ApiError};
use crate::shared::{models::GuestbookEntry, server_fns::ServerError};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use validator::Validate;

/// Request payload for creating a new guestbook entry.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "server", derive(Validate))]
pub struct CreateEntryRequest {
    /// The message content of the new guestbook entry.
    #[cfg_attr(
        feature = "server",
        validate(
            length(
                min = 1,
                max = 255,
                message = "Message must be between 1 and 255 characters"
            ),
            custom(
                function = "crate::backend::utils::validate_not_offensive",
                message = "watch you mouth"
            )
        )
    )]
    pub message: String,
    pub signature: Option<String>,
}

#[cfg(feature = "server")]
fn validation_message(errors: &validator::ValidationErrors) -> String {
    errors
        .field_errors()
        .get("message")
        .and_then(|errors| errors.first())
        .and_then(|error| error.message.as_deref())
        .unwrap_or("Invalid guestbook message")
        .to_owned()
}

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn submit_signature(
    mut payload: CreateEntryRequest,
) -> Result<GuestbookEntry, ServerError> {
    use crate::shared::models::NewGuestbookEntry;

    let guest = session.session.user.ok_or(ServerError::Unauthenticated)?;
    payload.message = payload.message.trim().to_string();
    payload
        .validate()
        .map_err(|errors| ServerError::Validation(validation_message(&errors)))?;

    let new_entry = NewGuestbookEntry {
        author_id: guest.id,
        author_username: guest.username,
        message: payload.message,
        signature: payload.signature,
    }
    .into();

    match state.guestbook_repo.create(&new_entry).await {
        Ok(entry) => Ok(entry),
        Err(ApiError::Database(sqlx::Error::Database(error))) if error.is_unique_violation() => {
            Err(ServerError::Conflict)
        }
        Err(error) => Err(ServerError::internal("create guestbook entry", error)),
    }
}

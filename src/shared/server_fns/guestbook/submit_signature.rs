//! Guestbook entry creation handler.
//!
//! This module contains the handler function for creating a new guestbook entry,
//! along with the necessary request payload structure.
#[cfg(feature = "server")]
use crate::backend::{
    AppState, domain::logic::SessionWrapper, errors::ApiError, repos::Repository,
};
use crate::shared::models::GuestbookEntry;
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
use dioxus::prelude::*;
#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn submit_signature(
    mut payload: CreateEntryRequest,
) -> Result<GuestbookEntry, ServerFnError> {
    use crate::shared::models::NewGuestbookEntry;

    let guest = session
        .session
        .user
        .ok_or_else(|| ServerFnError::ServerError {
            message: "Sign in before signing the guestbook".to_string(),
            code: 401,
            details: None,
        })?;
    payload.message = payload.message.trim().to_string();
    payload
        .validate()
        .map_err(|errors| ServerFnError::ServerError {
            message: errors
                .field_errors()
                .values()
                .flat_map(|errors| errors.iter())
                .find_map(|error| error.message.as_deref())
                .unwrap_or("Invalid guestbook entry")
                .to_string(),
            code: 400,
            details: None,
        })?;
    let new_entry = NewGuestbookEntry {
        author_id: guest.id,
        author_username: guest.username,
        message: payload.message,
        signature: payload.signature,
    }
    .into();
    match state.guestbook_repo.create(&new_entry).await {
        Ok(entry) => Ok(entry),
        Err(ApiError::DatabaseError(sqlx::Error::Database(error)))
            if error.is_unique_violation() =>
        {
            Err(ServerFnError::ServerError {
                message: "You have already signed the guestbook".to_string(),
                code: 409,
                details: None,
            })
        }
        Err(error) => Err(ServerFnError::new(error)),
    }
}

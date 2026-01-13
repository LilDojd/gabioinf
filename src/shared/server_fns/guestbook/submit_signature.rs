//! Guestbook entry creation handler.
//!
//! This module contains the handler function for creating a new guestbook entry,
//! along with the necessary request payload structure.
use crate::shared::models::{Guest, GuestbookEntry};
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
                message = "watch you language"
            )
        )
    )]
    pub message: String,
    pub signature: Option<String>,
}
use dioxus::prelude::*;
#[post("/submit_signature")]
pub async fn submit_signature(
    payload: CreateEntryRequest,
    guest: Guest,
) -> Result<Option<GuestbookEntry>, ServerFnError> {
    use crate::backend::AppState;
    use crate::backend::repos::Repository;
    use crate::shared::models::NewGuestbookEntry;
    payload
        .validate()
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let state = try_consume_context::<AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found in context"))?;
    let new_entry = NewGuestbookEntry {
        author_id: guest.id,
        author_username: guest.username,
        message: payload.message.trim().to_string(),
        signature: payload.signature,
    }
    .into();
    let entry = state
        .guestbook_repo
        .create(&new_entry)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(Some(entry))
}

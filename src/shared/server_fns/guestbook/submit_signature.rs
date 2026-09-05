//! Guestbook entry creation handler.
//!
//! This module contains the handler function for creating a new guestbook entry,
//! along with the necessary request payload structure.
#[cfg(feature = "server")]
use crate::backend::{AppState, auth::SessionWrapper, errors::ApiError};
use crate::shared::{models::GuestbookEntry, server_fns::ServerError};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use validator::Validate;

/// Request payload for creating a new guestbook entry.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
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
                function = "crate::backend::profanity::validate_no_severe_content",
                message = "Message contains offensive content"
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

#[cfg(feature = "server")]
fn validate_payload(mut payload: CreateEntryRequest) -> Result<CreateEntryRequest, ServerError> {
    payload.message = payload.message.trim().to_string();
    payload
        .validate()
        .map_err(|errors| ServerError::Validation(validation_message(&errors)))?;
    Ok(payload)
}

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn submit_signature(payload: CreateEntryRequest) -> Result<GuestbookEntry, ServerError> {
    use crate::shared::models::NewGuestbookEntry;

    let guest = session.session.user.ok_or(ServerError::Unauthenticated)?;
    let payload = validate_payload(payload)?;

    let new_entry = NewGuestbookEntry {
        author_id: guest.id,
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

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    fn request(message: &str) -> CreateEntryRequest {
        CreateEntryRequest {
            message: message.to_string(),
            signature: None,
        }
    }

    #[test]
    fn validation_trims_and_allows_clean_mild_and_moderate_messages() {
        for message in [
            "This is a clean message",
            "This is a bad word: crap",
            "F u c k",
        ] {
            let payload = validate_payload(request(&format!("  {message}  "))).unwrap();
            assert_eq!(payload.message, message);
        }
    }

    #[test]
    fn validation_rejects_severe_messages_with_user_facing_error() {
        assert_eq!(
            validate_payload(request("  i hope you die  ")),
            Err(ServerError::Validation(
                "Message contains offensive content".to_string()
            ))
        );
    }

    #[test]
    fn validation_preserves_message_length_limits() {
        for message in [String::new(), "  ".to_string(), "a".repeat(256)] {
            assert_eq!(
                validate_payload(request(&message)),
                Err(ServerError::Validation(
                    "Message must be between 1 and 255 characters".to_string()
                ))
            );
        }
        assert!(validate_payload(request("a")).is_ok());
        assert!(validate_payload(request(&format!("{}hello", "café ".repeat(50)))).is_ok());
    }
}

//! Guestbook entry creation handler.
//!
//! This module contains the handler function for creating a new guestbook entry,
//! along with the necessary request payload structure.
#[cfg(feature = "server")]
use crate::backend::{AppState, auth::AuthSession};
use crate::shared::{models::GuestbookEntry, server_fns::ServerError};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Request payload for creating a new guestbook entry.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CreateEntryRequest {
    /// The message content of the new guestbook entry.
    pub message: String,
    pub signature: Option<String>,
}

#[server(auth:AuthSession, state:axum::Extension<AppState>)]
pub async fn submit_signature(payload: CreateEntryRequest) -> Result<GuestbookEntry, ServerError> {
    use crate::shared::models::NewGuestbookEntry;

    let guest = auth.user().await.ok_or(ServerError::Unauthenticated)?;
    let payload = crate::backend::validation::guestbook_entry(payload)?;

    let new_entry = NewGuestbookEntry {
        author_id: guest.id,
        message: payload.message,
        signature: payload.signature,
    };

    match state.guestbook_repo.create(&new_entry).await {
        Ok(Some(entry)) => Ok(entry),
        Ok(None) => Err(ServerError::Conflict),
        Err(error) => Err(ServerError::internal("create guestbook entry", error)),
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use crate::backend::test_support::moderation_test_context;

    fn request(message: &str) -> CreateEntryRequest {
        CreateEntryRequest {
            message: message.to_string(),
            signature: None,
        }
    }

    #[sqlx::test]
    async fn authenticated_submission_rejects_severe_content_without_inserting(pool: sqlx::PgPool) {
        let context = moderation_test_context(pool.clone()).await;
        context
            .scope(async {
                assert_eq!(
                    submit_signature(request("i hope you die")).await,
                    Err(ServerError::Validation(
                        "Message contains offensive content".into()
                    ))
                );
                assert_eq!(
                    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM guestbook")
                        .fetch_one(&pool)
                        .await
                        .unwrap(),
                    0
                );

                let entry = submit_signature(request("  This is a bad word: crap  "))
                    .await
                    .unwrap();
                let stored: String =
                    sqlx::query_scalar("SELECT message FROM guestbook WHERE id = $1")
                        .bind(entry.id.as_value())
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                assert_eq!(stored, "This is a bad word: crap");
            })
            .await;
    }
}

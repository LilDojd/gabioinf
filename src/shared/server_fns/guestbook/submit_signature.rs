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

/// Build an authenticated request context for posting tests without GitHub or a running server.
#[cfg(all(test, feature = "server"))]
pub(crate) async fn moderation_test_context(
    pool: sqlx::PgPool,
) -> dioxus::fullstack::FullstackContext {
    use crate::{
        backend::auth::{AuthBackend, AuthSession, build_oauth_client},
        shared::models::{GithubId, Guest},
    };
    use axum::{
        extract::FromRequestParts,
        http::{Request, Response},
    };
    use dioxus::fullstack::FullstackContext;
    use std::{convert::Infallible, sync::Arc};
    use tower::{ServiceExt, service_fn};
    use tower_sessions::{MemoryStore, Session};

    let state = AppState::new(
        pool,
        "https://example.test".into(),
        axum_extra::extract::cookie::Key::generate(),
    );
    let guest = state
        .guest_repo
        .upsert(&Guest {
            github_id: GithubId(1),
            username: "moderation-test".into(),
            name: "Moderation Test".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    let backend = AuthBackend::new(
        state.guest_repo.clone(),
        build_oauth_client("test-client", "test-secret", &state.origin),
        reqwest::Client::new(),
    );
    // Let axum-login create its session extension, then retain the real request parts.
    let capture = service_fn(|request: Request<()>| async move {
        Ok::<_, Infallible>(Response::new(Some(request.into_parts().0)))
    });
    let mut request = Request::new(());
    request
        .extensions_mut()
        .insert(Session::new(None, Arc::new(MemoryStore::default()), None));
    let mut parts = axum_login::AuthManager::new(capture, backend, "moderation-test")
        .oneshot(request)
        .await
        .unwrap()
        .into_body()
        .unwrap();
    let mut session = AuthSession::from_request_parts(&mut parts, &())
        .await
        .unwrap();
    session.login(&guest).await.unwrap();
    parts.extensions.insert(session);
    parts.extensions.insert(state);
    FullstackContext::new(parts)
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

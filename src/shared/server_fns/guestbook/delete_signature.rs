#[cfg(feature = "server")]
use crate::backend::errors::ApiError;
#[cfg(feature = "server")]
use crate::backend::{domain::logic::SessionWrapper, repos::Repository, AppState};
use crate::shared::models::GuestbookEntry;
use dioxus::prelude::*;
#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn delete_signature(entry: GuestbookEntry) -> Result<(), ServerFnError> {
    match session.session.user {
        Some(user) if user.id == entry.author_id => {
            dioxus_logger::tracing::debug!("Deleting signature: {:?}", entry.id);
            state.guestbook_repo.delete(&entry).await.map_err(ServerFnError::new)
        }
        _ => {
            Err(ServerFnError::ServerError {
                message: ApiError::AuthorizationError("Unauthorized".to_string())
                    .to_string(),
                code: 403,
                details: None,
            })
        }
    }
}

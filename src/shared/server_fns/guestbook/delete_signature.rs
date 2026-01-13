#[cfg(feature = "server")]
use crate::backend::errors::ApiError;
#[cfg(feature = "server")]
use crate::backend::{domain::logic::SessionWrapper, repos::Repository, AppState};
use crate::shared::models::GuestbookEntry;
use dioxus::prelude::*;
#[server]
pub async fn delete_signature(entry: GuestbookEntry) -> Result<(), ServerFnError> {
    let session: SessionWrapper = extract().await?;
    match session.session.user {
        Some(user) if user.id == entry.author_id => {
            let FromContext(state): FromContext<AppState> = extract().await?;
            let guestbook_repo = state.guestbook_repo;
            dioxus_logger::tracing::debug!("Deleting signature: {:?}", entry.id);
            Ok(guestbook_repo.delete(&entry).await?)
        }
        _ => Err(ApiError::AuthorizationError("Unauthorized".to_string()).into()),
    }
}

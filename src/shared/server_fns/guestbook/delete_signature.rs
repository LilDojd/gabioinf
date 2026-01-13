#[cfg(feature = "server")]
use crate::backend::domain::logic::SessionWrapper;
use crate::shared::models::GuestbookEntry;
use dioxus::prelude::*;
#[delete("/delete_signature", session: SessionWrapper)]
pub async fn delete_signature(entry: GuestbookEntry) -> Result<(), ServerFnError> {
    use crate::backend::AppState;
    use crate::backend::errors::ApiError;
    use crate::backend::repos::Repository;
    match session.session.user {
        Some(user) if user.id == entry.author_id => {
            let state = try_consume_context::<AppState>()
                .ok_or_else(|| ServerFnError::new("AppState not found in context"))?;
            let guestbook_repo = state.guestbook_repo;
            dioxus_logger::tracing::debug!("Deleting signature: {:?}", entry.id);
            guestbook_repo
                .delete(&entry)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            Ok(())
        }
        _ => Err(ServerFnError::new(
            ApiError::AuthorizationError("Unauthorized".to_string()).to_string(),
        )),
    }
}

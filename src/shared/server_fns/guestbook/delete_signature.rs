#[cfg(feature = "server")]
use crate::backend::{AppState, domain::logic::SessionWrapper};
use crate::shared::models::GuestbookId;
use dioxus::prelude::*;

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn delete_signature(id: GuestbookId) -> Result<(), ServerFnError> {
    let user = session
        .session
        .user
        .ok_or_else(|| ServerFnError::ServerError {
            message: "Sign in before deleting a signature".to_string(),
            code: 401,
            details: None,
        })?;
    dioxus_logger::tracing::debug!("Deleting signature: {id:?}");
    let deleted = state
        .guestbook_repo
        .delete_owned(id, user.id)
        .await
        .map_err(ServerFnError::new)?;
    if deleted {
        Ok(())
    } else {
        Err(ServerFnError::ServerError {
            message: "Signature no longer exists".to_string(),
            code: 404,
            details: None,
        })
    }
}

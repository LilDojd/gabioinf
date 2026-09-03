#[cfg(feature = "server")]
use crate::backend::{AppState, auth::SessionWrapper};
use crate::shared::{models::GuestbookId, server_fns::ServerError};
use dioxus::prelude::*;

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn delete_signature(id: GuestbookId) -> Result<(), ServerError> {
    let user = session.session.user.ok_or(ServerError::Unauthenticated)?;
    tracing::debug!(?id, "Deleting signature");

    let deleted = state
        .guestbook_repo
        .delete_owned(id, user.id)
        .await
        .map_err(|error| ServerError::internal("delete guestbook entry", error))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerError::NotFound)
    }
}

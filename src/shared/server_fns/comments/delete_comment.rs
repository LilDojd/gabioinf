#[cfg(feature = "server")]
use crate::backend::{AppState, domain::logic::SessionWrapper};
use crate::shared::{models::CommentId, server_fns::ServerError};
use dioxus::prelude::*;

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn delete_comment(id: CommentId) -> Result<(), ServerError> {
    let user = session.session.user.ok_or(ServerError::Unauthenticated)?;
    let deleted = state
        .comment_repo
        .delete_owned(id, user.id)
        .await
        .map_err(|error| ServerError::internal("delete comment", error))?;

    if deleted {
        Ok(())
    } else {
        Err(ServerError::NotFound)
    }
}

#[cfg(feature = "server")]
use crate::backend::{AppState, auth::AuthSession};
use crate::shared::{models::CommentId, server_fns::ServerError};
use dioxus::prelude::*;

#[server(auth:AuthSession, state:axum::Extension<AppState>)]
pub async fn delete_comment(id: CommentId) -> Result<(), ServerError> {
    let user = auth.user().await.ok_or(ServerError::Unauthenticated)?;
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

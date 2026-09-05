#[cfg(feature = "server")]
use crate::backend::AppState;
use crate::shared::{models::Comment, server_fns::ServerError};
use dioxus::prelude::*;

#[server(state:axum::Extension<AppState>)]
pub async fn load_comments(slug: String) -> Result<Vec<Comment>, ServerError> {
    let rows = state
        .comment_repo
        .list(&slug)
        .await
        .map_err(|error| ServerError::internal("load comments", error))?;

    rows.into_iter()
        .map(|row| {
            super::render_comment(row)
                .map_err(|error| ServerError::internal("render stored comment", error))
        })
        .collect()
}

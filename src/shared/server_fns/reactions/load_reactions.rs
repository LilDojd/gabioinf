#[cfg(feature = "server")]
use crate::backend::{AppState, auth::AuthSession};
use crate::shared::{models::Reactions, server_fns::ServerError};
use dioxus::prelude::*;

#[server(auth:AuthSession, state:axum::Extension<AppState>)]
pub async fn load_reactions(slug: String) -> Result<Reactions, ServerError> {
    let viewer = auth.user().await.map(|user| user.id);
    state
        .reaction_repo
        .counts_for_post(&slug, viewer)
        .await
        .map_err(|error| ServerError::internal("load reactions", error))
}

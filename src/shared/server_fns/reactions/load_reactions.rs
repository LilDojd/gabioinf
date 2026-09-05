#[cfg(feature = "server")]
use crate::backend::{AppState, auth::SessionWrapper};
use crate::shared::{models::Reactions, server_fns::ServerError};
use dioxus::prelude::*;

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn load_reactions(slug: String) -> Result<Reactions, ServerError> {
    let viewer = session.session.user.map(|user| user.id);
    state
        .reaction_repo
        .counts_for_post(&slug, viewer)
        .await
        .map_err(|error| ServerError::internal("load reactions", error))
}

#[cfg(feature = "server")]
use crate::backend::{
    AppState,
    errors::ApiError,
    repos::{GuestbookEntryCriteria, Repository},
};
use crate::shared::{
    models::{Guest, GuestbookCursor, GuestbookEntry, GuestbookId, GuestbookPage},
    server_fns::ServerError,
};
use dioxus::prelude::*;

#[cfg(feature = "server")]
const SIGNATURES_PER_PAGE: usize = 10;

#[server(state:axum::Extension<AppState>)]
pub async fn load_signatures(
    cursor: Option<GuestbookCursor>,
    pinned_entry_id: Option<GuestbookId>,
) -> Result<GuestbookPage, ServerError> {
    let per_page = SIGNATURES_PER_PAGE - usize::from(cursor.is_none() && pinned_entry_id.is_some());
    state
        .guestbook_repo
        .read_page(cursor, per_page, pinned_entry_id)
        .await
        .map_err(|error| ServerError::internal("load guestbook entries", error))
}

#[server(state:axum::Extension<AppState>)]
pub async fn load_user_signature(user: Guest) -> Result<Option<GuestbookEntry>, ServerError> {
    let signature = state
        .guestbook_repo
        .read(&GuestbookEntryCriteria::WithAuthorId(user.id))
        .await;

    match signature {
        Ok(signature) => {
            dioxus_logger::tracing::info!("Found users signature");
            Ok(Some(signature))
        }
        Err(ApiError::DatabaseError(sqlx::Error::RowNotFound)) => {
            dioxus_logger::tracing::info!("User has not left a signature yet");
            Ok(None)
        }
        Err(error) => Err(ServerError::internal("load user guestbook entry", error)),
    }
}

#[cfg(feature = "server")]
use crate::backend::{
    AppState,
    errors::ApiError,
    repos::{GuestbookEntryCriteria, Repository},
};
use crate::shared::models::{Guest, GuestbookCursor, GuestbookEntry, GuestbookPage};
use dioxus::prelude::*;

const SIGNATURES_PER_PAGE: usize = 10;

#[server(state:axum::Extension<AppState>)]
pub async fn load_signatures(
    cursor: Option<GuestbookCursor>,
) -> Result<GuestbookPage, ServerFnError> {
    state
        .guestbook_repo
        .read_page(cursor, SIGNATURES_PER_PAGE)
        .await
        .map_err(ServerFnError::new)
}
#[server(state:axum::Extension<AppState>)]
pub async fn load_user_signature(user: Guest) -> Result<Option<GuestbookEntry>, ServerFnError> {
    let signature = state
        .guestbook_repo
        .read(&GuestbookEntryCriteria::WithAuthorId(user.id))
        .await;
    match signature {
        Ok(signature) => {
            dioxus_logger::tracing::info!("Found users signature");
            Ok(Some(signature))
        }
        Err(e) => match e {
            ApiError::DatabaseError(sqlx::Error::RowNotFound) => {
                dioxus_logger::tracing::info!("User has not left a signature yet");
                Ok(None)
            }
            _ => Err(ServerFnError::new(e)),
        },
    }
}

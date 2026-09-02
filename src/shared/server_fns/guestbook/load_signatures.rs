#[cfg(feature = "server")]
use crate::backend::{
    AppState,
    errors::ApiError,
    repos::{GuestbookEntryCriteria, Repository},
};
use crate::shared::models::{Guest, GuestbookEntry};
use dioxus::prelude::*;
#[server(state:axum::Extension<AppState>)]
pub async fn load_signatures(
    page: u32,
    per_page: usize,
) -> Result<Vec<GuestbookEntry>, ServerFnError> {
    let signatures = state
        .guestbook_repo
        .read_page(page, per_page)
        .await
        .map_err(ServerFnError::new)?;
    Ok(signatures)
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

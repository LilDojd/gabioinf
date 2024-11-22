#[cfg(feature = "server")]
use crate::backend::{
    errors::ApiError,
    repos::{GuestbookEntryCriteria, Repository},
    AppState,
};
use crate::shared::models::{Guest, GuestbookEntry};
use dioxus::prelude::*;
#[server(LoadSignatures)]
pub async fn load_signatures(
    page: u32,
    per_page: usize,
) -> Result<Vec<GuestbookEntry>, ServerFnError> {
    let FromContext(state): FromContext<AppState> = extract().await?;
    let guestbook_repo = state.guestbook_repo;
    let signatures = guestbook_repo.read_page(page, per_page).await?;
    Ok(signatures)
}
#[server(LoadUserSignature)]
pub async fn load_user_signature(user: Guest) -> Result<Option<GuestbookEntry>, ServerFnError> {
    let FromContext(state): FromContext<AppState> = extract().await?;
    let guestbook_repo = state.guestbook_repo;
    let signature = guestbook_repo
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
            _ => Err(ServerFnError::ServerError(e.to_string())),
        },
    }
}

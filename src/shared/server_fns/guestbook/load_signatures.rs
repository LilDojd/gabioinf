use crate::shared::models::{Guest, GuestbookEntry};
use dioxus::prelude::*;
#[get("/load_signatures?page&per_page")]
pub async fn load_signatures(
    page: u32,
    per_page: usize,
) -> Result<Vec<GuestbookEntry>, ServerFnError> {
    use crate::backend::AppState;
    let state = try_consume_context::<AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found in context"))?;
    let guestbook_repo = state.guestbook_repo;
    let signatures = guestbook_repo
        .read_page(page, per_page)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(signatures)
}
#[post("/load_user_signature")]
pub async fn load_user_signature(user: Guest) -> Result<Option<GuestbookEntry>, ServerFnError> {
    use crate::backend::AppState;
    use crate::backend::errors::ApiError;
    use crate::backend::repos::{GuestbookEntryCriteria, Repository};
    let state = try_consume_context::<AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found in context"))?;
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
            _ => Err(ServerFnError::new(e.to_string())),
        },
    }
}

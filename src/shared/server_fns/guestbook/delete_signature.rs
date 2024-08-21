#[cfg(feature = "server")]
use crate::backend::{repos::Repository, AppState};
use crate::shared::models::GuestbookEntry;
use dioxus::prelude::*;
#[server(DeleteSignature)]
pub async fn delete_signature(entry: GuestbookEntry) -> Result<(), ServerFnError> {
    let FromContext(state): FromContext<AppState> = extract().await?;
    let guestbook_repo = state.guestbook_repo;
    dioxus_logger::tracing::debug!("Deleting signature: {:?}", entry.id);
    Ok(guestbook_repo.delete(&entry).await?)
}

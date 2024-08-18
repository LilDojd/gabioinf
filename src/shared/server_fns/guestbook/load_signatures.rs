use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::backend::AppState;
use crate::shared::models::GuestbookEntry;

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

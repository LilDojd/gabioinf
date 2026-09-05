#[cfg(feature = "server")]
use crate::backend::{AppState, auth::SessionWrapper};
use crate::{
    auth::AuthState,
    shared::{
        models::{GuestbookCursor, GuestbookId, GuestbookPage},
        server_fns::ServerError,
    },
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

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn load_guestbook_user() -> Result<AuthState, ServerError> {
    let Some(guest) = session.session.user else {
        return Ok(AuthState::Unauthenticated);
    };
    let entry = state
        .guestbook_repo
        .find_by_author(guest.id)
        .await
        .map_err(|error| ServerError::internal("load user guestbook entry", error))?;
    Ok(AuthState::Authenticated(Box::new(crate::auth::UserState {
        guest,
        entry,
    })))
}

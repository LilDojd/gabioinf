#[cfg(feature = "server")]
use crate::backend::domain::logic::SessionWrapper;
use crate::shared::models::{Guest, GuestId};
use dioxus::prelude::*;
#[get("/get_user", session: SessionWrapper)]
pub async fn get_user() -> Result<Option<Guest>, ServerFnError> {
    match session.session.user {
        Some(user) => Ok(Some(user)),
        None => Ok(None),
    }
}
#[get("/get_user_by_id?{id}", session: SessionWrapper)]
pub async fn get_user_by_id(id: GuestId) -> Result<Option<Guest>, ServerFnError> {
    match session.session.user {
        Some(user) if user.id == id => Ok(Some(user)),
        _ => Ok(None),
    }
}

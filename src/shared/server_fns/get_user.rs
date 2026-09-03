#[cfg(feature = "server")]
use crate::backend::auth::SessionWrapper;
use crate::shared::{models::Guest, server_fns::ServerError};
use dioxus::prelude::*;
#[server(session:SessionWrapper)]
pub async fn get_user() -> Result<Option<Guest>, ServerError> {
    match session.session.user {
        Some(user) => Ok(Some(user)),
        None => Ok(None),
    }
}

#[cfg(feature = "server")]
use crate::backend::auth::AuthSession;
use crate::shared::{models::Guest, server_fns::ServerError};
use dioxus::prelude::*;
#[server(auth:AuthSession)]
pub async fn get_user() -> Result<Option<Guest>, ServerError> {
    match auth.user().await {
        Some(user) => Ok(Some(user)),
        None => Ok(None),
    }
}

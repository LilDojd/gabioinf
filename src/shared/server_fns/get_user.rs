use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::backend::domain::logic::SessionWrapper;
use crate::shared::models::Guest;

#[server(GetUserName)]
pub async fn get_user() -> Result<Option<Guest>, ServerFnError> {
    let session: SessionWrapper = extract().await?;

    match session.session.user {
        Some(user) => Ok(Some(user)),
        None => Ok(None),
    }
}

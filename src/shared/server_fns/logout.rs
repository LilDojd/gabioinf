#[cfg(feature = "server")]
use crate::backend::auth::SessionWrapper;
use crate::shared::server_fns::ServerError;
use dioxus::prelude::*;
#[server(session:SessionWrapper)]
pub async fn logout() -> Result<(), ServerError> {
    let mut session = session;
    tracing::info!("Logging out");
    session
        .session
        .logout()
        .await
        .map_err(|error| ServerError::internal("log out", error))?;
    Ok(())
}

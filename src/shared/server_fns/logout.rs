#[cfg(feature = "server")]
use crate::backend::auth::AuthSession;
use crate::shared::server_fns::ServerError;
use dioxus::prelude::*;
#[server(auth:AuthSession)]
pub async fn logout() -> Result<(), ServerError> {
    tracing::info!("Logging out");
    auth.logout()
        .await
        .map_err(|error| ServerError::internal("log out", error))?;
    Ok(())
}

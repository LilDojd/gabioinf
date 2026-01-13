#[cfg(feature = "server")]
use crate::backend::domain::logic::SessionWrapper;
use dioxus::prelude::*;
#[post("/logout", mut session: SessionWrapper)]
pub async fn logout() -> Result<(), ServerFnError> {
    dioxus_logger::tracing::info!("Logging out");
    session
        .session
        .logout()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[cfg(feature = "server")]
use crate::backend::domain::logic::SessionWrapper;
use dioxus::prelude::*;
#[server(session:SessionWrapper)]
pub async fn logout() -> Result<(), ServerFnError> {
    let mut session = session;
    dioxus_logger::tracing::info!("Logging out");
    session.session.logout().await.map_err(ServerFnError::new)?;
    Ok(())
}

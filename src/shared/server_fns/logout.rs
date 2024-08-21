#[cfg(feature = "server")]
use crate::backend::domain::logic::SessionWrapper;
use dioxus::prelude::*;
#[server(Logout)]
pub async fn logout() -> Result<(), ServerFnError> {
    let mut session: SessionWrapper = extract().await?;
    dioxus_logger::tracing::info!("Logging out");
    session.session.logout().await?;
    Ok(())
}

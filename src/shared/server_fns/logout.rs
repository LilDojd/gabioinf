use dioxus::prelude::*;
#[cfg(feature = "server")]
use crate::backend::domain::logic::SessionWrapper;
#[server(Logout)]
pub async fn logout() -> Result<(), ServerFnError> {
    let mut session: SessionWrapper = extract().await?;
    dioxus_logger::tracing::info!("Logging out");
    session.session.logout().await?;
    Ok(())
}

use crate::backend::domain::logic::{oauth::CSRF_STATE_KEY, AuthSession};
use axum::{
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Router,
};
use axum_login::tower_sessions::Session;
use serde::Deserialize;
pub const NEXT_URL_KEY: &str = "auth.next-url";
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}
pub fn router() -> Router<()> {
    Router::new().route("/login", get(self::get::login))
}
mod get {
    use super::*;
    pub async fn login(
        auth_session: AuthSession,
        session: Session,
        Form(NextUrl { next }): Form<NextUrl>,
    ) -> impl IntoResponse {
        dioxus_logger::tracing::info!("Hit login route");
        let (auth_url, csrf_state) = auth_session.backend.authorize_url_unscoped();
        session
            .insert(CSRF_STATE_KEY, csrf_state.secret())
            .await
            .expect("Serialization should not fail.");
        session
            .insert(NEXT_URL_KEY, next)
            .await
            .expect("Serialization should not fail.");
        Redirect::to(auth_url.as_str())
    }
}

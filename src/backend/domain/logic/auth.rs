use crate::backend::domain::logic::{oauth::CSRF_STATE_KEY, AuthSession};
use axum::{
    http::StatusCode,
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
    Router::new()
        .route("/login", get(self::post::login))
        .route("/logout", get(self::get::logout))
}
mod post {
    use super::*;
    pub async fn login(
        auth_session: AuthSession,
        session: Session,
        Form(NextUrl { next }): Form<NextUrl>,
    ) -> impl IntoResponse {
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
mod get {
    use super::*;
    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

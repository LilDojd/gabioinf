//! Debug-only fixture sign-in; production builds contain neither this module nor its route.

use super::{AuthSession, login::local_path};
use axum::{
    Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DevLogin {
    username: String,
    next: Option<String>,
}

pub(super) fn router() -> Router<()> {
    Router::new().route("/dev-login", get(dev_login))
}

async fn dev_login(
    mut auth_session: AuthSession,
    Query(DevLogin { username, next }): Query<DevLogin>,
) -> Response {
    let user = match auth_session
        .backend
        .guest_repo
        .find_by_username(&username)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, "could not load the development user");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if let Err(error) = auth_session.login(&user).await {
        tracing::error!(%error, "could not create the development session");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let next = next
        .as_deref()
        .and_then(local_path)
        .unwrap_or_else(|| "/".to_string());
    Redirect::to(&next).into_response()
}

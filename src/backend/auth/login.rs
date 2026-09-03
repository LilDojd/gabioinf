//! `GET /v1/login?next=/path`: starts the GitHub OAuth flow.
//!
//! The PKCE verifier, CSRF state and the `next` path are parked in the session
//! until GitHub redirects back to `/v1/oauth/callback` (see [`super::callback`]).
use super::{AuthSession, callback::PENDING_AUTHORIZATION_KEY};
use axum::{
    Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum_login::tower_sessions::Session;
use serde::Deserialize;

pub const NEXT_URL_KEY: &str = "auth.next-url";

#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub(super) fn router() -> Router<()> {
    Router::new().route("/login", get(login))
}

async fn login(
    auth_session: AuthSession,
    session: Session,
    Query(NextUrl { next }): Query<NextUrl>,
) -> impl IntoResponse {
    let (auth_url, pending_authorization) = auth_session.backend.authorize_url_unscoped();
    let next = next.as_deref().and_then(local_path);
    let stored = session
        .insert(PENDING_AUTHORIZATION_KEY, pending_authorization)
        .await
        .and(session.insert(NEXT_URL_KEY, next).await);
    match stored {
        Ok(()) => Redirect::to(auth_url.as_str()).into_response(),
        Err(error) => {
            tracing::error!(%error, "could not store the pending sign-in");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Only same-site absolute paths may be used as a post-login redirect, so the
/// `next` parameter cannot be abused as an open redirect (`//evil.com`, `https://…`).
pub(crate) fn local_path(candidate: &str) -> Option<String> {
    let is_local = candidate.starts_with('/')
        && !candidate.starts_with("//")
        && !candidate.starts_with("/\\")
        && !candidate.chars().any(char::is_control);
    is_local.then(|| candidate.to_string())
}

#[cfg(test)]
mod tests {
    use super::local_path;

    #[test]
    fn next_must_be_a_same_site_path() {
        assert_eq!(local_path("/guestbook").as_deref(), Some("/guestbook"));
        assert_eq!(
            local_path("/blog/post?x=1#c").as_deref(),
            Some("/blog/post?x=1#c")
        );
        assert_eq!(local_path("//evil.com"), None);
        assert_eq!(local_path("/\\evil.com"), None);
        assert_eq!(local_path("https://evil.com"), None);
        assert_eq!(local_path("guestbook"), None);
        assert_eq!(local_path("/a\r\nSet-Cookie: x"), None);
    }
}

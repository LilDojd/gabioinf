//! `GET /v1/oauth/callback`: the second half of the GitHub OAuth flow.
use super::{AuthSession, Credentials, login::NEXT_URL_KEY};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{
    Router,
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl, basic::BasicClient};
use oauth2::{CsrfToken, EndpointNotSet, EndpointSet, PkceCodeVerifier, RedirectUrl};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
pub const PENDING_AUTHORIZATION_KEY: &str = "oauth.pending-authorization";
pub(crate) type SetOauthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;
#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}
#[derive(Deserialize, Serialize)]
pub(crate) struct PendingAuthorization {
    pub(crate) csrf_state: CsrfToken,
    pub(crate) pkce_verifier: PkceCodeVerifier,
}
pub(super) fn router() -> Router<()> {
    Router::new().route("/oauth/callback", get(self::get::callback))
}
/// The GitHub OAuth client; `origin` is where GitHub sends visitors back to.
pub fn build_oauth_client(client_id: &str, client_secret: &str, origin: &str) -> SetOauthClient {
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("the GitHub authorization URL is valid");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("the GitHub token URL is valid");
    let oauth_redirect_uri = RedirectUrl::new(format!("{origin}/v1/oauth/callback"))
        .expect("the configured origin forms a valid callback URL");
    tracing::debug!("OAuth redirect URI: {}", oauth_redirect_uri);
    BasicClient::new(ClientId::new(client_id.to_owned()))
        .set_client_secret(ClientSecret::new(client_secret.to_owned()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(oauth_redirect_uri)
}
async fn take_pending_authorization(
    session: &Session,
) -> Result<Option<PendingAuthorization>, tower_sessions::session::Error> {
    session.remove(PENDING_AUTHORIZATION_KEY).await
}
mod get {
    use super::*;
    use crate::backend::auth::login::local_path;

    /// `GET /v1/oauth/callback?code=…&state=…`: GitHub sends the visitor back here.
    ///
    /// Failures are logged with their cause; the visitor only sees a short status
    /// page because nothing here is actionable for them.
    pub async fn callback(
        mut auth_session: AuthSession,
        session: Session,
        Query(AuthzResp {
            code,
            state: new_state,
        }): Query<AuthzResp>,
    ) -> Response {
        let pending = match take_pending_authorization(&session).await {
            Ok(Some(pending)) => pending,
            Ok(None) => {
                return failure(
                    StatusCode::BAD_REQUEST,
                    "sign-in expired or was not started here; try again",
                );
            }
            Err(error) => {
                tracing::error!(%error, "could not read the pending sign-in");
                return failure(StatusCode::INTERNAL_SERVER_ERROR, "sign-in failed");
            }
        };
        let creds = Credentials {
            code,
            old_state: pending.csrf_state,
            new_state,
            pkce_verifier: pending.pkce_verifier,
        };
        let user = match auth_session.authenticate(creds).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                tracing::warn!("OAuth state mismatch during sign-in");
                return failure(
                    StatusCode::UNAUTHORIZED,
                    "sign-in state mismatch; try again",
                );
            }
            Err(error) => {
                tracing::error!(%error, "GitHub sign-in failed");
                return failure(
                    StatusCode::BAD_GATEWAY,
                    "GitHub sign-in failed; try again later",
                );
            }
        };
        if let Err(error) = auth_session.login(&user).await {
            tracing::error!(%error, "could not create the session");
            return failure(StatusCode::INTERNAL_SERVER_ERROR, "sign-in failed");
        }
        let next = session
            .remove::<String>(NEXT_URL_KEY)
            .await
            .ok()
            .flatten()
            .and_then(|next| local_path(&next))
            .unwrap_or_else(|| "/".to_string());
        Redirect::to(&next).into_response()
    }

    fn failure(status: StatusCode, message: &'static str) -> Response {
        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oauth2::PkceCodeChallenge;
    use std::sync::Arc;
    use tower_sessions::MemoryStore;

    #[tokio::test]
    async fn pkce_verifier_is_taken_once() {
        let session = Session::new(None, Arc::new(MemoryStore::default()), None);
        let (_, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let pending = PendingAuthorization {
            csrf_state: CsrfToken::new_random(),
            pkce_verifier,
        };
        session
            .insert(PENDING_AUTHORIZATION_KEY, pending)
            .await
            .unwrap();

        assert!(
            take_pending_authorization(&session)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            take_pending_authorization(&session)
                .await
                .unwrap()
                .is_none()
        );
    }
}

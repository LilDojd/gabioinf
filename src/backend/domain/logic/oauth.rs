//! Handles OAuth flow, user auth and session management
use crate::backend::domain::logic::auth::NEXT_URL_KEY;
use crate::backend::domain::logic::AuthSession;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{
    extract::Query, response::{IntoResponse, Redirect},
    Router,
};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use oauth2::{CsrfToken, EndpointNotSet, EndpointSet, RedirectUrl};
use serde::Deserialize;
use tower_sessions::Session;
pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";
pub(crate) type SetOauthClient = BasicClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;
#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}
pub fn router() -> Router<()> {
    Router::new().route("/oauth/callback", get(self::get::callback))
}
/// Builds an OAuth2 client for the GitHub OAuth provider
/// # Arguments
///
/// * `client_id` - The client ID for the OAuth application
/// * `client_secret` - The client secret for the OAuth application
/// * `domain` - The domain name which will be used for redirect URI for the OAuth application
///
/// # Returns
///
/// A `BasicClient` object for the GitHub OAuth provider
///
pub fn build_oauth_client<S: AsRef<str>>(
    client_id: S,
    client_secret: S,
    domain: S,
) -> SetOauthClient {
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new(
            "https://github.com/login/oauth/access_token".to_string(),
        )
        .expect("Invalid token endpoint URL");
    let oauth_redirect_uri = if domain.as_ref().contains("localhost:") {
        let port = domain.as_ref().split(':').next_back().unwrap().split('/').next().unwrap();
        RedirectUrl::new(format!("http://localhost:{port}/v1/oauth/callback"))
    } else {
        RedirectUrl::new(format!("https://{}/v1/oauth/callback", domain.as_ref()))
    }
        .expect("Invalid redirect URL");
    dioxus_logger::tracing::debug!("OAuth redirect URI: {}", oauth_redirect_uri);
    BasicClient::new(ClientId::new(client_id.as_ref().to_owned()))
        .set_client_secret(ClientSecret::new(client_secret.as_ref().to_owned()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(oauth_redirect_uri)
}
mod get {
    use super::*;
    use crate::backend::domain::models::Credentials;
    pub async fn callback(
        mut auth_session: AuthSession,
        session: Session,
        Query(AuthzResp { code, state: new_state }): Query<AuthzResp>,
    ) -> impl IntoResponse {
        let Ok(Some(old_state)) = session.get(CSRF_STATE_KEY).await else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        let creds = Credentials {
            code,
            old_state,
            new_state,
        };
        let user = match auth_session.authenticate(creds).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return StatusCode::UNAUTHORIZED.into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
            Redirect::to(&next).into_response()
        } else {
            Redirect::to("/").into_response()
        }
    }
}

//! Handles OAuth flow, user auth and session management

use crate::domain::logic::AuthSession;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
    Router,
};

use crate::domain::logic::auth::{LoginTemplate, NEXT_URL_KEY};

use oauth2::CsrfToken;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};

use serde::Deserialize;
use tower_sessions::Session;

pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";

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
/// * `oauth_redirect_uri` - The redirect URI for the OAuth application
///
/// # Returns
///
/// A `BasicClient` object for the GitHub OAuth provider
///
pub fn build_oauth_client<S: AsRef<str>>(client_id: S, client_secret: S) -> BasicClient {
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id.as_ref().to_owned()),
        Some(ClientSecret::new(client_secret.as_ref().to_owned())),
        auth_url,
        Some(token_url),
    )
}

use askama::Template;

#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    username: &'a str,
}

pub async fn protected(auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.user {
        Some(user) => ProtectedTemplate {
            username: &user.username,
        }
        .into_response(),

        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

mod get {

    use crate::domain::models::Credentials;

    use super::*;

    pub async fn callback(
        mut auth_session: AuthSession,
        session: Session,
        Query(AuthzResp {
            code,
            state: new_state,
        }): Query<AuthzResp>,
    ) -> impl IntoResponse {
        tracing::error!("Callback");
        let Ok(Some(old_state)) = session.get(CSRF_STATE_KEY).await else {
            tracing::error!("CSRF issue");

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
                tracing::error!("CSRF issue 2");
                return (
                    StatusCode::UNAUTHORIZED,
                    LoginTemplate {
                        message: Some("Invalid CSRF state.".to_string()),
                        next: None,
                    },
                )
                    .into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
            Redirect::to("/").into_response()
        } else {
            Redirect::to("/").into_response()
        }
    }
}

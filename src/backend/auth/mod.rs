//! GitHub sign-in: the axum-login backend plus the `/v1/login` and
//! `/v1/oauth/callback` handlers.
//!
//! Flow: `/v1/login` stores a PKCE verifier + CSRF state in the session and sends
//! the visitor to GitHub; GitHub redirects back to the callback, which exchanges
//! the code for a token, fetches the GitHub profile, upserts the guest row and
//! logs the session in.

mod callback;
#[cfg(debug_assertions)]
mod dev_login;
mod login;

pub use callback::build_oauth_client;
use callback::{PendingAuthorization, SetOauthClient};

use crate::{
    backend::{
        errors::{ApiError, BResult},
        repos::GuestRepo,
    },
    shared::models::{Guest, GuestId, NewGuest},
};
use axum::{Router, extract::FromRequestParts, http::request::Parts};
use axum_login::{AuthUser, AuthnBackend, UserId};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
    http::header::{AUTHORIZATION, USER_AGENT},
};
use reqwest::Url;

/// The fixture sign-in route is compiled and registered only in debug builds.
pub fn router() -> Router<()> {
    let router = Router::new()
        .merge(login::router())
        .merge(callback::router());
    #[cfg(debug_assertions)]
    let router = router.merge(dev_login::router());
    router
}

impl AuthUser for Guest {
    type Id = GuestId;
    fn id(&self) -> Self::Id {
        self.id
    }
    /// Changing this invalidates existing sessions; there is no secret per user,
    /// so the username is as good as it gets.
    fn session_auth_hash(&self) -> &[u8] {
        self.username.as_bytes()
    }
}

/// What the OAuth callback hands to [`AuthBackend::authenticate`].
#[derive(Debug)]
pub struct Credentials {
    pub code: String,
    /// CSRF state stored when the flow started.
    pub old_state: CsrfToken,
    /// CSRF state GitHub sent back; must equal `old_state`.
    pub new_state: CsrfToken,
    pub pkce_verifier: PkceCodeVerifier,
}

#[derive(Clone, Debug)]
pub struct AuthBackend {
    guest_repo: GuestRepo,
    client: SetOauthClient,
    reqwest_client: reqwest::Client,
}
impl AuthBackend {
    pub fn new(
        guest_repo: GuestRepo,
        client: SetOauthClient,
        reqwest_client: reqwest::Client,
    ) -> Self {
        Self {
            guest_repo,
            client,
            reqwest_client,
        }
    }
    pub fn authorize_url<I>(&self, scopes: I) -> (Url, PendingAuthorization)
    where
        I: IntoIterator<Item = Scope>,
    {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, csrf_state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes)
            .set_pkce_challenge(pkce_challenge)
            .url();
        (
            url,
            PendingAuthorization {
                csrf_state,
                pkce_verifier,
            },
        )
    }
    pub fn authorize_url_unscoped(&self) -> (Url, PendingAuthorization) {
        self.authorize_url(std::iter::empty())
    }
}
impl AuthnBackend for AuthBackend {
    type User = Guest;
    type Credentials = Credentials;
    type Error = ApiError;
    async fn authenticate(&self, creds: Self::Credentials) -> BResult<Option<Self::User>> {
        if creds.old_state.secret() != creds.new_state.secret() {
            return Ok(None);
        }
        tracing::debug!("Received OAuth callback");
        // oauth2 is pinned to reqwest 0.12, so it cannot share `self.reqwest_client` (0.13).
        // Redirects are disabled as the oauth2 docs recommend, to rule out SSRF via the token endpoint.
        let token_client = oauth2::reqwest::ClientBuilder::new()
            .redirect(oauth2::reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| Self::Error::Authentication(error.to_string()))?;
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(creds.code))
            .set_pkce_verifier(creds.pkce_verifier)
            .request_async(&token_client)
            .await
            .map_err(|error| Self::Error::Authentication(describe_token_error(&error)))?;
        tracing::debug!("Getting user data from GitHub API");
        let response = self
            .reqwest_client
            .get("https://api.github.com/user")
            .header(USER_AGENT.as_str(), "ga-guestbook")
            .header(
                AUTHORIZATION.as_str(),
                format!("Bearer {}", token.access_token().secret()),
            )
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?
            .error_for_status()?;
        let github_user = response.json::<NewGuest>().await?;
        tracing::debug!("Received user data from GitHub: {:?}", github_user);
        let guest = self.guest_repo.upsert(&github_user.into()).await?;
        Ok(Some(guest))
    }
    async fn get_user(&self, user_id: &UserId<Self>) -> BResult<Option<Self::User>> {
        self.guest_repo.find_by_id(*user_id).await
    }
}
/// GitHub answers `200 OK` with `{"error": …}` instead of a proper OAuth error
/// response, which oauth2 reports as an opaque parse failure. Surface the body so
/// the log says *why* (`bad_verification_code`, `incorrect_client_credentials`, …).
fn describe_token_error<E: std::error::Error, R: oauth2::ErrorResponse>(
    error: &oauth2::RequestTokenError<E, R>,
) -> String {
    match error {
        oauth2::RequestTokenError::Parse(_, body) => {
            format!("token exchange failed: {}", String::from_utf8_lossy(body))
        }
        other => format!("token exchange failed: {other}"),
    }
}

pub type AuthSession = axum_login::AuthSession<AuthBackend>;
#[derive(Debug, Clone)]
pub struct SessionWrapper {
    pub session: AuthSession,
}
#[derive(Debug)]
pub struct StateError;
impl std::error::Error for StateError {}
impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(internal) state error")
    }
}
impl axum::response::IntoResponse for StateError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::status::StatusCode::INTERNAL_SERVER_ERROR,
            "(internal) state error",
        )
            .into_response()
    }
}
impl<S> FromRequestParts<S> for SessionWrapper
where
    S: Send + Sync,
{
    type Rejection = StateError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = AuthSession::from_request_parts(parts, state).await;
        match session {
            Ok(session) => Ok(Self { session }),
            Err(_) => Err(StateError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    fn test_backend() -> AuthBackend {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/gabioinf")
            .unwrap();
        AuthBackend::new(
            GuestRepo::new(pool),
            build_oauth_client("client-id", "client-secret", "https://example.com"),
            reqwest::Client::new(),
        )
    }

    #[tokio::test]
    async fn authorization_url_uses_fresh_matching_s256_pkce_challenges() {
        let backend = test_backend();
        let (first_url, first_pending) = backend.authorize_url_unscoped();
        let (second_url, _) = backend.authorize_url_unscoped();
        let first_challenge = first_url
            .query_pairs()
            .find(|(key, _)| key == "code_challenge")
            .map(|(_, value)| value.into_owned())
            .unwrap();
        let second_challenge = second_url
            .query_pairs()
            .find(|(key, _)| key == "code_challenge")
            .map(|(_, value)| value.into_owned())
            .unwrap();
        let expected = PkceCodeChallenge::from_code_verifier_sha256(&first_pending.pkce_verifier);

        assert_eq!(first_challenge, expected.as_str());
        assert_ne!(first_challenge, second_challenge);
        assert!(
            first_url
                .query_pairs()
                .any(|(key, value)| key == "code_challenge_method" && value == "S256")
        );
    }
}

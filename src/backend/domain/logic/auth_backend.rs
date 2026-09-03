use crate::{
    backend::{
        domain::models::Credentials,
        errors::{ApiError, BResult},
        repos::GuestRepo,
    },
    shared::models::{Guest, NewGuest},
};
use axum_login::{AuthnBackend, UserId};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope, TokenResponse,
    http::header::{AUTHORIZATION, USER_AGENT},
};
use reqwest::Url;
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
        dioxus_logger::tracing::debug!("Received OAuth callback");
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(creds.code))
            .set_pkce_verifier(creds.pkce_verifier)
            .request_async(&oauth2::reqwest::Client::new())
            .await
            .map_err(|error| Self::Error::Authentication(error.to_string()))?;
        dioxus_logger::tracing::debug!("Getting user data from GitHub API");
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
            .await;
        let github_user = response?.json::<NewGuest>().await?;
        dioxus_logger::tracing::debug!("Received user data from GitHub: {:?}", github_user);
        let guest = self.guest_repo.upsert(&github_user.into()).await?;
        Ok(Some(guest))
    }
    async fn get_user(&self, user_id: &UserId<Self>) -> BResult<Option<Self::User>> {
        self.guest_repo.find_by_id(*user_id).await
    }
}
pub type AuthSession = axum_login::AuthSession<AuthBackend>;
#[derive(Debug, Clone)]
pub struct SessionWrapper {
    pub session: AuthSession,
}
use super::oauth::{PendingAuthorization, SetOauthClient};
use axum::{extract::FromRequestParts, http::request::Parts};
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
    use crate::backend::domain::logic::oauth::build_oauth_client;
    use sqlx::postgres::PgPoolOptions;

    fn test_backend() -> AuthBackend {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/gabioinf")
            .unwrap();
        AuthBackend::new(
            GuestRepo::new(pool),
            build_oauth_client("client-id", "client-secret", "example.com"),
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

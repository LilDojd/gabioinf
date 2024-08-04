use crate::backend::{
    domain::models::{Credentials, Guest, NewGuest, PermissionTargets},
    errors::{ApiError, BResult},
    repos::{GroupsAndPermissionsRepo, GuestCriteria, PgRepository, Repository},
};
use axum_login::{AuthnBackend, AuthzBackend, UserId};
use oauth2::{
    basic::BasicClient, http::header::{AUTHORIZATION, USER_AGENT},
    reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope, TokenResponse,
};
use reqwest::Url;
use std::collections::HashSet;
#[derive(Clone, Debug)]
pub struct AuthBackend {
    guest_repo: PgRepository<Guest>,
    gp_repo: GroupsAndPermissionsRepo,
    client: BasicClient,
}
impl AuthBackend {
    pub fn new(
        guest_repo: PgRepository<Guest>,
        gp_repo: GroupsAndPermissionsRepo,
        client: BasicClient,
    ) -> Self {
        Self {
            guest_repo,
            gp_repo,
            client,
        }
    }
    pub fn authorize_url<I>(&self, scopes: I) -> (Url, CsrfToken)
    where
        I: IntoIterator<Item = Scope>,
    {
        self.client.authorize_url(CsrfToken::new_random).add_scopes(scopes).url()
    }
    pub fn authorize_url_unscoped(&self) -> (Url, CsrfToken) {
        self.client.authorize_url(CsrfToken::new_random).url()
    }
}
#[axum::async_trait]
impl AuthnBackend for AuthBackend {
    type User = Guest;
    type Credentials = Credentials;
    type Error = ApiError;
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> BResult<Option<Self::User>> {
        if creds.old_state.secret() != creds.new_state.secret() {
            return Ok(None);
        }
        tracing::debug!("Received OAuth callback");
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(creds.code))
            .request_async(async_http_client)
            .await
            .map_err(|e| Self::Error::AuthenticationError(e.to_string()))?;
        tracing::debug!("Getting user data from GitHub API");
        let github_user = reqwest::Client::new()
            .get("https://api.github.com/user")
            .header(USER_AGENT.as_str(), "GABioInf-Guestbook")
            .header(
                AUTHORIZATION.as_str(),
                format!("Bearer {}", token.access_token().secret()),
            )
            .send()
            .await?
            .json::<NewGuest>()
            .await?;
        tracing::debug!("Received user data from GitHub: {:?}", github_user);
        let guest = self.guest_repo.create(&github_user.into()).await?;
        Ok(Some(guest))
    }
    async fn get_user(&self, user_id: &UserId<Self>) -> BResult<Option<Self::User>> {
        self.guest_repo.read(&GuestCriteria::WithGuestId(*user_id)).await.map(Some)
    }
}
#[axum::async_trait]
impl AuthzBackend for AuthBackend {
    type Permission = PermissionTargets;
    async fn get_user_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let perms = self.gp_repo.get_user_specific_permissions(user.id).await?;
        Ok(perms.into_iter().collect())
    }
    async fn get_group_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let perms = self.gp_repo.get_user_group_permissions(user.id).await?;
        Ok(perms.into_iter().collect())
    }
    async fn get_all_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let perms = self.gp_repo.get_all_user_permissions(user.id).await?;
        Ok(perms.into_iter().collect())
    }
}
pub type AuthSession = axum_login::AuthSession<AuthBackend>;

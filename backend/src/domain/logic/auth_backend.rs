use std::collections::HashSet;

use axum_login::{AuthnBackend, AuthzBackend, UserId};
use oauth2::{
    basic::BasicClient,
    http::header::{AUTHORIZATION, USER_AGENT},
    reqwest::async_http_client,
    AuthorizationCode, CsrfToken, Scope, TokenResponse,
};
use reqwest::Url;
use serde::Deserialize;

use crate::{
    domain::models::{Guest, NewGuest},
    errors::{ApiError, BResult},
    repos::{GuestCriteria, PgRepository, Repository},
};

#[derive(Clone, Debug)]
pub struct AuthBackend {
    repo: PgRepository<Guest>,
    client: BasicClient,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
}

impl AuthBackend {
    pub fn new(repo: PgRepository<Guest>, client: BasicClient) -> Self {
        Self { repo, client }
    }

    pub fn authorize_url<I>(&self, scopes: I) -> (Url, CsrfToken)
    where
        I: IntoIterator<Item = Scope>,
    {
        self.client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes)
            .url()
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

    async fn authenticate(&self, creds: Self::Credentials) -> BResult<Option<Self::User>> {
        // Ensure the CSFR state matches
        if creds.old_state.secret() != creds.new_state.secret() {
            return Err(Self::Error::AuthorizationError(
                "CSRF state mismatch".to_string(),
            ));
        }

        // Exchange code
        tracing::debug!("Received OAuth callback");
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(creds.code))
            .request_async(async_http_client)
            .await
            .map_err(|e| Self::Error::ExternalServiceError(e.to_string()))?;

        // Request user info
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

        // Add to db
        let guest = self.repo.create(&github_user.into()).await?;

        Ok(Some(guest))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> BResult<Option<Self::User>> {
        self.repo
            .read(&GuestCriteria::WithGuestId(*user_id))
            .await
            .map(Some)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PermissionTargets {
    AddSignature,
    DeleteOwnSignature,
    DeleteAnySignature,
    EditOwnSignature,
    MarkAsNaughty,
}

impl Into<PermissionTargets> for String {
    fn into(self) -> PermissionTargets {
        match self.as_str() {
            "leavesignature" => PermissionTargets::AddSignature,
            "deletesignature" => PermissionTargets::DeleteOwnSignature,
            "deleteanysignature" => PermissionTargets::DeleteAnySignature,
            "editsignature" => PermissionTargets::EditOwnSignature,
            "markasnaughty" => PermissionTargets::MarkAsNaughty,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Permission {
    name: PermissionTargets,
}

#[axum::async_trait]
impl AuthzBackend for AuthBackend {
    type Permission = Permission;

    // async fn get_user_permissions(
    //     &self,
    //     user: &Self::User,
    // ) -> Result<HashSet<Self::Permission>, Self::Error> {
    //     let conn = self.get()?;
    //     let user = user.clone();
    //
    //     let permissions = conn.prepare(
    //         "SELECT DISTINCT permissions.name FROM users, permissions, user_permissions WHERE users.id = ?1 AND users.id = user_permissions.userid AND user_permissions.permissionid = permissions.id",
    //     )?
    //         .query_map_into([user.id])?
    //         .collect::<Result<HashSet<_>, _>>()?;
    //     Ok(permissions)
    // }

    async fn get_group_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let permissions = sqlx::query_as!(
            Self::Permission,
            "
            SELECT DISTINCT permissions.name
            FROM guests
            JOIN guests_groups on guests.id = guests_groups.guest_id
            JOIN groups_permissions on guests_groups.group_id = groups_permissions.group_id
            JOIN permissions on groups_permissions.permission_id = permissions.id
            WHERE guests.id = $1
            ",
            user.id.as_value()
        )
        .fetch_all(&self.repo.pool)
        .await?;

        Ok(permissions.into_iter().collect())
    }
}

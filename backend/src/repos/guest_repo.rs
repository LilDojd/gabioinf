
use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::{GithubId, Guest, GuestId},
    errors::{ApiError, BResult},
    AppState,
};

use super::{PgRepository, Repository};

#[derive(Debug, Serialize, Deserialize)]
pub enum GuestCriteria {
    WithGuestId(GuestId),
    WithGithubId(GithubId),
    Latest,
}

#[axum::async_trait]
impl Repository<Guest> for PgRepository<Guest> {
    type Error = ApiError;
    type Criteria = GuestCriteria;

    async fn read_all(&self) -> BResult<Vec<Guest>> {
        let guests = sqlx::query_as!(Guest, "SELECT * FROM guests ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;
        Ok(guests)
    }
    async fn read(&self, criteria: &Self::Criteria) -> BResult<Guest> {
        let guest = match criteria {
            GuestCriteria::WithGuestId(id) => {
                sqlx::query_as!(Guest, "SELECT * FROM guests WHERE id = $1", id.as_value())
                    .fetch_one(&self.pool)
                    .await?
            }
            GuestCriteria::WithGithubId(github_id) => {
                sqlx::query_as!(
                    Guest,
                    "SELECT * FROM guests WHERE github_id = $1",
                    github_id.as_value()
                )
                .fetch_one(&self.pool)
                .await?
            }
            GuestCriteria::Latest => {
                sqlx::query_as!(Guest, "SELECT * FROM guests ORDER BY created_at DESC")
                    .fetch_one(&self.pool)
                    .await?
            }
        };
        Ok(guest)
    }

    async fn create(&self, guest: &Guest) -> BResult<Guest> {
        let created_guest = sqlx::query_as!(
            Guest,
            "INSERT INTO guests (github_id, name, username) 
             VALUES ($1, $2, $3) 
             ON CONFLICT (github_id) DO UPDATE 
             SET name = EXCLUDED.name, username = EXCLUDED.username 
             RETURNING *",
            guest.github_id.as_value(),
            guest.name,
            guest.username,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(created_guest)
    }

    async fn update(&self, guest: &Guest) -> BResult<Guest> {
        let updated_guest = sqlx::query_as!(
            Guest,
            "UPDATE guests
            SET name = $2, username = $3, is_naughty = $4, is_admin = $5, naughty_reason = $6, updated_at = NOW()
            WHERE id = $1
            RETURNING *",
            guest.id.as_value(),
            guest.name,
            guest.username,
            guest.is_naughty,
            guest.is_admin,
            guest.naughty_reason
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(updated_guest)
    }

    async fn delete(&self, guest: &Guest) -> BResult<()> {
        sqlx::query!("DELETE FROM guests WHERE id = $1", guest.id.as_value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

impl PgRepository<Guest> {
    pub async fn get_all_admins(&self) -> Result<Vec<Guest>, ApiError> {
        let admins = sqlx::query_as!(Guest, "SELECT * FROM guests WHERE is_admin = true")
            .fetch_all(&self.pool)
            .await?;
        Ok(admins)
    }

    pub async fn get_all_naughty_bois(&self) -> Result<Vec<Guest>, ApiError> {
        let naughty_guests = sqlx::query_as!(
            Guest,
            "SELECT * FROM guests WHERE is_naughty = true ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(naughty_guests)
    }

    pub async fn flag_as_naughty<S: AsRef<str>>(
        &self,
        id: i64,
        reason: S,
    ) -> Result<Guest, ApiError> {
        let flagged_guest = sqlx::query_as!(
            Guest,
            r#"
            UPDATE guests
            SET is_naughty = true, naughty_reason = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            reason.as_ref()
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(flagged_guest)
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Guest {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Guest>()
            .cloned()
            .ok_or(ApiError::AuthorizationError(
                "User is not authenticated".to_string(),
            ))
    }
}

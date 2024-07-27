use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::{GithubId, Guest, GuestId},
    errors::{ApiError, BResult},
    AppState,
};

use super::{PgRepository, Repository};

/// Criteria for querying guest data.
#[derive(Debug, Serialize, Deserialize)]
pub enum GuestCriteria {
    /// Query by guest ID.
    WithGuestId(GuestId),
    /// Query by GitHub ID.
    WithGithubId(GithubId),
    /// Query for the latest guest.
    Latest,
}

#[axum::async_trait]
impl Repository<Guest> for PgRepository<Guest> {
    type Error = ApiError;
    type Criteria = GuestCriteria;

    /// Retrieves all guests, ordered by creation date.
    async fn read_all(&self) -> BResult<Vec<Guest>> {
        let guests = sqlx::query_as!(Guest, "SELECT * FROM guests ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;
        Ok(guests)
    }

    /// Retrieves a single guest based on the provided criteria.
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

    /// Creates a new guest or updates an existing one if there's a conflict on github_id.
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

    /// Updates an existing guest's information.
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

    /// Deletes a guest from the database.
    async fn delete(&self, guest: &Guest) -> BResult<()> {
        sqlx::query!("DELETE FROM guests WHERE id = $1", guest.id.as_value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

impl PgRepository<Guest> {
    /// Retrieves all guests with admin privileges.
    pub async fn get_all_admins(&self) -> Result<Vec<Guest>, ApiError> {
        let admins = sqlx::query_as!(Guest, "SELECT * FROM guests WHERE is_admin = true")
            .fetch_all(&self.pool)
            .await?;
        Ok(admins)
    }

    /// Retrieves all guests marked as naughty, ordered by creation date descending.
    pub async fn get_all_naughty_bois(&self) -> Result<Vec<Guest>, ApiError> {
        let naughty_guests = sqlx::query_as!(
            Guest,
            "SELECT * FROM guests WHERE is_naughty = true ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(naughty_guests)
    }

    /// Flags a guest as naughty and sets the reason.
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

    /// Extracts a [`Guest`] instance from the request parts.
    ///
    /// This implementation allows `Guest` to be used as an extractor in Axum handlers.
    /// It retrieves the `Guest` from the request extensions, which should be set by
    /// an authentication middleware.
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
use super::{PgRepository, Repository};
use crate::backend::{
    errors::{ApiError, BResult},
    AppState,
};
use crate::shared::models::{GithubId, Guest, GuestId};
use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
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
            "INSERT INTO guests (github_id, name, username, access_token) 
             VALUES ($1, $2, $3, $4) 
             ON CONFLICT (github_id) DO UPDATE 
             SET access_token = excluded.access_token 
             RETURNING *",
            guest.github_id.as_value(),
            guest.name,
            guest.username,
            guest.access_token
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
            SET name = $2, username = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING *",
            guest.id.as_value(),
            guest.name,
            guest.username,
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
impl PgRepository<Guest> {}
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
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    #[sqlx::test]
    async fn test_create_and_read_guest(pool: PgPool) {
        let repo = PgRepository::<Guest>::new(pool);
        let guest = Guest {
            github_id: GithubId(12345),
            name: "Test User".to_string(),
            username: "testuser".to_string(),
            ..Default::default()
        };
        let created_guest = repo.create(&guest).await.unwrap();
        assert_eq!(created_guest.github_id, guest.github_id);
        assert_eq!(created_guest.name, guest.name);
        let read_guest = repo
            .read(&GuestCriteria::WithGithubId(guest.github_id))
            .await
            .unwrap();
        assert_eq!(read_guest.id, created_guest.id);
        assert_eq!(read_guest.github_id, guest.github_id);
    }
    #[sqlx::test]
    async fn test_update_guest(pool: PgPool) {
        let repo = PgRepository::<Guest>::new(pool);
        let mut guest = Guest {
            github_id: GithubId(67890),
            name: "Update Test".to_string(),
            username: "updatetest".to_string(),
            ..Default::default()
        };
        let created_guest = repo.create(&guest).await.unwrap();
        guest.id = created_guest.id;
        guest.name = "Updated Name".to_string();
        let updated_guest = repo.update(&guest).await.unwrap();
        assert_eq!(updated_guest.name, "Updated Name");
    }
    #[sqlx::test]
    async fn test_delete_guest(pool: PgPool) {
        let repo = PgRepository::<Guest>::new(pool);
        let guest = Guest {
            github_id: GithubId(11111),
            name: "Delete Test".to_string(),
            username: "deletetest".to_string(),
            ..Default::default()
        };
        let created_guest = repo.create(&guest).await.unwrap();
        repo.delete(&created_guest).await.unwrap();
        let result = repo
            .read(&GuestCriteria::WithGithubId(guest.github_id))
            .await;
        assert!(result.is_err());
    }
}

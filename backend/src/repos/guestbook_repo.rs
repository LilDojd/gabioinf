use super::{PgRepository, Repository};
use crate::domain::models::{GuestId, GuestbookEntry, GuestbookId};
use crate::errors::{ApiError, BResult};
use serde::{Deserialize, Serialize};
/// Criteria for querying guestbook entries.
#[derive(Debug, Serialize, Deserialize)]
pub enum GuestbookEntryCriteria {
    /// Query by guestbook entry ID.
    WithId(GuestbookId),
    /// Query by author ID.
    WithAuthorId(GuestId),
    /// Query for the latest entry.
    Latest,
}
#[axum::async_trait]
impl Repository<GuestbookEntry> for PgRepository<GuestbookEntry> {
    type Error = ApiError;
    type Criteria = GuestbookEntryCriteria;
    /// Retrieves all guestbook entries, ordered by creation date descending.
    async fn read_all(&self) -> BResult<Vec<GuestbookEntry>> {
        let entries = sqlx::query_as!(
            GuestbookEntry,
            "SELECT * FROM guestbook ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }
    /// Retrieves a single guestbook entry based on the provided criteria.
    async fn read(&self, criteria: &Self::Criteria) -> BResult<GuestbookEntry> {
        let entry = match criteria {
            GuestbookEntryCriteria::WithId(id) => {
                sqlx::query_as!(
                    GuestbookEntry,
                    "SELECT * FROM guestbook WHERE id = $1",
                    id.as_value()
                )
                .fetch_one(&self.pool)
                .await?
            }
            GuestbookEntryCriteria::WithAuthorId(author_id) => {
                sqlx::query_as!(
                    GuestbookEntry,
                    "SELECT * FROM guestbook WHERE author_id = $1",
                    author_id.as_value()
                )
                .fetch_one(&self.pool)
                .await?
            }
            GuestbookEntryCriteria::Latest => {
                sqlx::query_as!(
                    GuestbookEntry,
                    "SELECT * FROM guestbook ORDER BY created_at DESC"
                )
                .fetch_one(&self.pool)
                .await?
            }
        };
        Ok(entry)
    }
    /// Creates a new guestbook entry.
    async fn create(&self, entry: &GuestbookEntry) -> BResult<GuestbookEntry> {
        let created_entry = sqlx::query_as!(
            GuestbookEntry,
            r#"
            INSERT INTO guestbook (message, signature, author_id)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            entry.message,
            entry.signature,
            entry.author_id.as_value(),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(created_entry)
    }
    /// Updates an existing guestbook entry.
    async fn update(&self, entry: &GuestbookEntry) -> BResult<GuestbookEntry> {
        let updated_entry = sqlx::query_as!(
            GuestbookEntry,
            r#"
            UPDATE guestbook
            SET message = $2, signature = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            entry.id.as_value(),
            entry.message,
            entry.signature,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(updated_entry)
    }
    /// Deletes a guestbook entry.
    async fn delete(&self, entry: &GuestbookEntry) -> BResult<()> {
        sqlx::query!("DELETE FROM guestbook WHERE id = $1", entry.id.as_value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
impl PgRepository<GuestbookEntry> {}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::setup_guest;
    use sqlx::PgPool;
    #[sqlx::test]
    #[should_panic]
    async fn create_entry_without_user(pool: PgPool) {
        let repo = PgRepository::<GuestbookEntry>::new(pool);
        let entry = GuestbookEntry {
            message: "Test message".to_string(),
            signature: Some("Test signature".to_string()),
            author_id: GuestId(0),
            ..Default::default()
        };
        repo.create(&entry).await.unwrap();
    }
    #[sqlx::test]
    async fn test_create_and_read_entry(pool: PgPool) {
        setup_guest(&pool).await;
        let repo = PgRepository::<GuestbookEntry>::new(pool.clone());
        let entry = GuestbookEntry {
            message: "Test message".to_string(),
            signature: Some("Test signature".to_string()),
            author_id: GuestId(1),
            ..Default::default()
        };
        let created_entry = repo.create(&entry).await.unwrap();
        assert_eq!(created_entry.message, entry.message);
        assert_eq!(created_entry.signature, entry.signature);
        let read_entry = repo
            .read(&GuestbookEntryCriteria::WithId(created_entry.id))
            .await
            .unwrap();
        assert_eq!(read_entry.id, created_entry.id);
        assert_eq!(read_entry.message, entry.message);
    }
    #[sqlx::test]
    async fn test_update_entry(pool: PgPool) {
        setup_guest(&pool).await;
        let repo = PgRepository::<GuestbookEntry>::new(pool);
        let mut entry = GuestbookEntry {
            message: "Original message".to_string(),
            signature: Some("Original signature".to_string()),
            author_id: GuestId(1),
            ..Default::default()
        };
        let created_entry = repo.create(&entry).await.unwrap();
        entry.id = created_entry.id;
        entry.message = "Updated message".to_string();
        let updated_entry = repo.update(&entry).await.unwrap();
        assert_eq!(updated_entry.message, "Updated message");
    }
    #[sqlx::test]
    async fn test_delete_entry(pool: PgPool) {
        setup_guest(&pool).await;
        let repo = PgRepository::<GuestbookEntry>::new(pool);
        let entry = GuestbookEntry {
            message: "Delete test".to_string(),
            signature: None,
            author_id: GuestId(1),
            ..Default::default()
        };
        let created_entry = repo.create(&entry).await.unwrap();
        repo.delete(&created_entry).await.unwrap();
        let result = repo
            .read(&GuestbookEntryCriteria::WithId(created_entry.id))
            .await;
        assert!(result.is_err());
    }
}

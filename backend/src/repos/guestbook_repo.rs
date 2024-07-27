use super::{PgRepository, Repository};
use crate::domain::models::{GuestId, GuestbookEntry, GuestbookId};
use crate::errors::{ApiError, BResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum GuestbookEntryCriteria {
    WithId(GuestbookId),
    WithAuthorId(GuestId),
    Latest,
}

#[axum::async_trait]
impl Repository<GuestbookEntry> for PgRepository<GuestbookEntry> {
    type Error = ApiError;
    type Criteria = GuestbookEntryCriteria;

    async fn read_all(&self) -> BResult<Vec<GuestbookEntry>> {
        let entries = sqlx::query_as!(
            GuestbookEntry,
            "SELECT * FROM guestbook ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

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

    async fn delete(&self, entry: &GuestbookEntry) -> BResult<()> {
        sqlx::query!("DELETE FROM guestbook WHERE id = $1", entry.id.as_value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

impl PgRepository<GuestbookEntry> {}

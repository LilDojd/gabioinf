#![allow(unused)]
use super::{PgRepository, Repository};
use crate::backend::errors::{ApiError, BResult};
use crate::shared::models::{GuestId, GuestbookCursor, GuestbookEntry, GuestbookId, GuestbookPage};
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
            INSERT INTO guestbook (message, signature, author_id, author_username)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            entry.message,
            entry.signature,
            entry.author_id.as_value(),
            entry.author_username,
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
impl PgRepository<GuestbookEntry> {
    pub async fn read_page(
        &self,
        cursor: Option<GuestbookCursor>,
        per_page: usize,
    ) -> BResult<GuestbookPage> {
        let per_page = per_page.clamp(1, 100);
        let limit = per_page as i64 + 1;
        let mut entries = if let Some(cursor) = cursor {
            sqlx::query_as!(
                GuestbookEntry,
                r#"
                SELECT * FROM guestbook
                WHERE (created_at, id) < ($1, $2)
                ORDER BY created_at DESC, id DESC
                LIMIT $3
                "#,
                cursor.created_at,
                cursor.id.as_value(),
                limit,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                GuestbookEntry,
                "SELECT * FROM guestbook ORDER BY created_at DESC, id DESC LIMIT $1",
                limit,
            )
            .fetch_all(&self.pool)
            .await?
        };
        let has_more = entries.len() > per_page;
        entries.truncate(per_page);
        let next_cursor = has_more.then(|| {
            let last = entries
                .last()
                .expect("a page with an extra row cannot be empty");
            GuestbookCursor {
                created_at: last.created_at,
                id: last.id,
            }
        });
        Ok(GuestbookPage {
            entries,
            next_cursor,
        })
    }

    pub async fn delete_owned(&self, id: GuestbookId, author_id: GuestId) -> BResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM guestbook WHERE id = $1 AND author_id = $2",
            id.as_value(),
            author_id.as_value(),
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::utils::setup_guest,
        shared::models::{GithubId, Guest},
    };
    use sqlx::PgPool;

    async fn create_guest(pool: &PgPool, number: i64) -> Guest {
        PgRepository::<Guest>::new(pool.clone())
            .create(&Guest {
                github_id: GithubId(number),
                name: format!("Test User {number}"),
                username: format!("testuser{number}"),
                ..Default::default()
            })
            .await
            .unwrap()
    }

    async fn create_entry_for(
        repo: &PgRepository<GuestbookEntry>,
        guest: &Guest,
    ) -> GuestbookEntry {
        repo.create(&GuestbookEntry {
            message: format!("Message from {}", guest.username),
            author_id: guest.id,
            author_username: guest.username.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
    }
    #[sqlx::test]
    #[should_panic]
    async fn create_entry_without_user(pool: PgPool) {
        let repo = PgRepository::<GuestbookEntry>::new(pool);
        let entry = GuestbookEntry {
            message: "Test message".to_string(),
            signature: Some("Test signature".to_string()),
            author_id: GuestId(0),
            author_username: "testuser".to_string(),
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
            author_username: "testuser".to_string(),
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
            author_username: "testuser".to_string(),
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
            author_username: "testuser".to_string(),
            ..Default::default()
        };
        let created_entry = repo.create(&entry).await.unwrap();
        repo.delete(&created_entry).await.unwrap();
        let result = repo
            .read(&GuestbookEntryCriteria::WithId(created_entry.id))
            .await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn read_page_cursor_is_stable_when_new_entries_arrive(pool: PgPool) {
        let repo = PgRepository::<GuestbookEntry>::new(pool.clone());
        let mut old_entries = Vec::new();
        for number in 1..=3 {
            let guest = create_guest(&pool, number).await;
            old_entries.push(create_entry_for(&repo, &guest).await);
        }
        let created_at = time::OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        sqlx::query!("UPDATE guestbook SET created_at = $1", created_at)
            .execute(&pool)
            .await
            .unwrap();

        let first = repo.read_page(None, 2).await.unwrap();
        assert_eq!(first.entries.len(), 2);
        assert!(first.entries[0].id.as_value() > first.entries[1].id.as_value());
        let first_ids = first
            .entries
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>();

        let new_guest = create_guest(&pool, 4).await;
        let new_entry = create_entry_for(&repo, &new_guest).await;
        let second = repo.read_page(first.next_cursor, 2).await.unwrap();

        assert_eq!(second.entries.len(), 1);
        assert_eq!(second.entries[0].id, old_entries[0].id);
        assert!(!first_ids.contains(&second.entries[0].id));
        assert_ne!(second.entries[0].id, new_entry.id);
        assert!(second.next_cursor.is_none());
    }

    #[sqlx::test]
    async fn delete_owned_cannot_delete_another_users_entry(pool: PgPool) {
        let owner = create_guest(&pool, 1).await;
        let other = create_guest(&pool, 2).await;
        let repo = PgRepository::<GuestbookEntry>::new(pool.clone());
        let entry = create_entry_for(&repo, &owner).await;

        assert!(!repo.delete_owned(entry.id, other.id).await.unwrap());
        assert!(
            repo.read(&GuestbookEntryCriteria::WithId(entry.id))
                .await
                .is_ok()
        );
        assert!(repo.delete_owned(entry.id, owner.id).await.unwrap());
    }
}

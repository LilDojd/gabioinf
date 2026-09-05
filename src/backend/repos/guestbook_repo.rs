use crate::backend::errors::{ApiError, BResult};
use crate::shared::models::{GuestId, GuestbookCursor, GuestbookEntry, GuestbookId, GuestbookPage};

#[derive(Debug, Clone)]
pub struct GuestbookRepo {
    pool: sqlx::PgPool,
}

impl GuestbookRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, entry: &GuestbookEntry) -> BResult<GuestbookEntry> {
        Ok(sqlx::query_as!(
            GuestbookEntry,
            r#"
            WITH g AS (
                INSERT INTO guestbook (message, signature, author_id)
                VALUES ($1, $2, $3)
                RETURNING *
            )
            SELECT
                g.id AS "id: GuestbookId",
                g.message,
                g.signature,
                g.created_at,
                g.updated_at,
                g.author_id AS "author_id: GuestId",
                u.username AS author_username
            FROM g JOIN guests u ON u.id = g.author_id
            "#,
            entry.message,
            entry.signature,
            entry.author_id.as_value(),
        )
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn find_by_author(&self, author_id: GuestId) -> BResult<Option<GuestbookEntry>> {
        Ok(sqlx::query_as!(
            GuestbookEntry,
            r#"
            SELECT
                g.id AS "id: GuestbookId",
                g.message,
                g.signature,
                g.created_at,
                g.updated_at,
                g.author_id AS "author_id: GuestId",
                u.username AS author_username
            FROM guestbook g JOIN guests u ON u.id = g.author_id
            WHERE g.author_id = $1
            "#,
            author_id.as_value(),
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn read_page(
        &self,
        cursor: Option<GuestbookCursor>,
        per_page: usize,
        excluded_id: Option<GuestbookId>,
    ) -> BResult<GuestbookPage> {
        let per_page = per_page.clamp(1, 100);
        let limit = per_page as i64 + 1 + i64::from(excluded_id.is_some());
        let mut entries = if let Some(cursor) = cursor {
            sqlx::query_as!(
                GuestbookEntry,
                r#"
                SELECT
                    g.id AS "id: GuestbookId",
                    g.message,
                    g.signature,
                    g.created_at,
                    g.updated_at,
                    g.author_id AS "author_id: GuestId",
                    u.username AS author_username
                FROM guestbook g JOIN guests u ON u.id = g.author_id
                WHERE (g.created_at, g.id) < ($1, $2)
                ORDER BY g.created_at DESC, g.id DESC
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
                r#"
                SELECT
                    g.id AS "id: GuestbookId",
                    g.message,
                    g.signature,
                    g.created_at,
                    g.updated_at,
                    g.author_id AS "author_id: GuestId",
                    u.username AS author_username
                FROM guestbook g JOIN guests u ON u.id = g.author_id
                ORDER BY g.created_at DESC, g.id DESC
                LIMIT $1
                "#,
                limit,
            )
            .fetch_all(&self.pool)
            .await?
        };
        if let Some(excluded_id) = excluded_id {
            entries.retain(|entry| entry.id != excluded_id);
        }
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
            total: self.total().await?,
        })
    }

    /// Every signature in the guestbook, not only the ones on the current page.
    async fn total(&self) -> BResult<usize> {
        let total = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "total!" FROM guestbook"#)
            .fetch_one(&self.pool)
            .await?;
        usize::try_from(total).map_err(|_| ApiError::InvalidData("guestbook count exceeds usize"))
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
        backend::repos::GuestRepo,
        shared::models::{GithubId, Guest},
    };
    use sqlx::PgPool;

    async fn create_guest(pool: &PgPool, number: i64) -> Guest {
        GuestRepo::new(pool.clone())
            .upsert(&Guest {
                github_id: GithubId(number),
                name: format!("Test User {number}"),
                username: format!("testuser{number}"),
                ..Default::default()
            })
            .await
            .unwrap()
    }

    async fn create_entry_for(repo: &GuestbookRepo, guest: &Guest) -> GuestbookEntry {
        repo.create(&GuestbookEntry {
            message: format!("Message from {}", guest.username),
            author_id: guest.id,
            ..Default::default()
        })
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn create_and_find_entry_by_author(pool: PgPool) {
        let guest = create_guest(&pool, 1).await;
        let repo = GuestbookRepo::new(pool);
        let entry = create_entry_for(&repo, &guest).await;

        assert_eq!(repo.find_by_author(guest.id).await.unwrap(), Some(entry));
    }

    #[sqlx::test]
    async fn read_page_cursor_is_stable_when_new_entries_arrive(pool: PgPool) {
        let repo = GuestbookRepo::new(pool.clone());
        let mut old_entries = Vec::new();
        for number in 1..=3 {
            let guest = create_guest(&pool, number).await;
            old_entries.push(create_entry_for(&repo, &guest).await);
        }
        let created_at = time::OffsetDateTime::from_unix_timestamp(1_700_000_000)
            .expect("test timestamp is valid");
        sqlx::query!("UPDATE guestbook SET created_at = $1", created_at)
            .execute(&pool)
            .await
            .unwrap();

        let first = repo.read_page(None, 2, None).await.unwrap();
        assert_eq!(first.entries.len(), 2);
        assert!(first.entries[0].id.as_value() > first.entries[1].id.as_value());
        let first_ids = first
            .entries
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>();

        let new_guest = create_guest(&pool, 4).await;
        let new_entry = create_entry_for(&repo, &new_guest).await;
        let second = repo.read_page(first.next_cursor, 2, None).await.unwrap();

        assert_eq!(second.entries.len(), 1);
        assert_eq!(second.entries[0].id, old_entries[0].id);
        assert!(!first_ids.contains(&second.entries[0].id));
        assert_ne!(second.entries[0].id, new_entry.id);
        assert!(second.next_cursor.is_none());
    }

    #[sqlx::test]
    async fn read_page_excludes_pinned_entry_without_shortening_pages(pool: PgPool) {
        let repo = GuestbookRepo::new(pool.clone());
        let mut entries = Vec::new();
        for number in 1..=12 {
            let guest = create_guest(&pool, number).await;
            entries.push(create_entry_for(&repo, &guest).await);
        }
        let pinned_id = entries[5].id;

        let first = repo.read_page(None, 9, Some(pinned_id)).await.unwrap();
        assert_eq!(first.entries.len(), 9);
        assert!(first.entries.iter().all(|entry| entry.id != pinned_id));
        // The total covers the whole guestbook: rows past this page and the pinned one.
        assert_eq!(first.total, 12);

        let second = repo
            .read_page(first.next_cursor, 10, Some(pinned_id))
            .await
            .unwrap();
        assert_eq!(second.entries.len(), 2);
        assert!(second.entries.iter().all(|entry| entry.id != pinned_id));
        assert_eq!(second.total, 12);
        assert!(second.next_cursor.is_none());
    }

    #[sqlx::test]
    async fn delete_owned_cannot_delete_another_users_entry(pool: PgPool) {
        let owner = create_guest(&pool, 1).await;
        let other = create_guest(&pool, 2).await;
        let repo = GuestbookRepo::new(pool);
        let entry = create_entry_for(&repo, &owner).await;

        assert!(!repo.delete_owned(entry.id, other.id).await.unwrap());
        let found = repo.find_by_author(owner.id).await.unwrap();
        assert_eq!(found.as_ref(), Some(&entry));
        assert!(repo.delete_owned(entry.id, owner.id).await.unwrap());
        assert!(repo.find_by_author(owner.id).await.unwrap().is_none());
    }
}

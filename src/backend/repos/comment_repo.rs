use crate::{
    backend::errors::BResult,
    shared::models::{CommentId, GithubId, GuestId},
};
use time::OffsetDateTime;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CommentRow {
    pub id: CommentId,
    pub parent_id: Option<CommentId>,
    pub username: String,
    pub github_id: GithubId,
    pub body: String,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct CommentRepo {
    pool: sqlx::PgPool,
}

impl CommentRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn list(&self, slug: &str) -> BResult<Vec<CommentRow>> {
        Ok(sqlx::query_as!(
            CommentRow,
            r#"
            SELECT
                c.id AS "id: CommentId",
                c.parent_id AS "parent_id?: CommentId",
                g.username,
                g.github_id AS "github_id: GithubId",
                c.body,
                c.created_at
            FROM comments c
            JOIN guests g ON g.id = c.author_id
            WHERE c.post_slug = $1
            ORDER BY c.created_at, c.id
            "#,
            slug,
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub(crate) async fn create(
        &self,
        slug: &str,
        author_id: GuestId,
        parent_id: Option<CommentId>,
        body: &str,
    ) -> BResult<Option<CommentRow>> {
        let parent_id = parent_id.map(|id| id.0);
        Ok(sqlx::query_as!(
            CommentRow,
            r#"
            WITH inserted AS (
                INSERT INTO comments (post_slug, author_id, parent_id, body)
                SELECT $1::VARCHAR(80), $2::BIGINT, $3::BIGINT, $4::VARCHAR(2000)
                WHERE $3::BIGINT IS NULL OR EXISTS (
                    SELECT 1 FROM comments
                    WHERE id = $3::BIGINT
                      AND post_slug = $1::VARCHAR(80)
                      AND parent_id IS NULL
                )
                RETURNING id, parent_id, author_id, body, created_at
            )
            SELECT
                i.id AS "id: CommentId",
                i.parent_id AS "parent_id?: CommentId",
                g.username,
                g.github_id AS "github_id: GithubId",
                i.body,
                i.created_at
            FROM inserted i
            JOIN guests g ON g.id = i.author_id
            "#,
            slug,
            author_id.as_value(),
            parent_id,
            body,
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub(crate) async fn post_slug(&self, id: CommentId) -> BResult<Option<String>> {
        Ok(
            sqlx::query_scalar!("SELECT post_slug FROM comments WHERE id = $1", id.0)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn delete_owned(&self, id: CommentId, author_id: GuestId) -> BResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM comments WHERE id = $1 AND author_id = $2",
            id.0,
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

    #[sqlx::test]
    async fn list_orders_comments_and_joins_authors(pool: PgPool) {
        let first_author = create_guest(&pool, 1).await;
        let second_author = create_guest(&pool, 2).await;
        let repo = CommentRepo::new(pool);
        let root = repo
            .create("post", first_author.id, None, "Root")
            .await
            .unwrap()
            .unwrap();
        let reply = repo
            .create("post", second_author.id, Some(root.id), "Reply")
            .await
            .unwrap()
            .unwrap();
        repo.create("another-post", first_author.id, None, "Hidden")
            .await
            .unwrap();

        let comments = repo.list("post").await.unwrap();

        assert_eq!(comments, vec![root, reply]);
        assert_eq!(comments[0].username, first_author.username);
        assert_eq!(comments[1].github_id, second_author.github_id);
    }

    #[sqlx::test]
    async fn replies_are_limited_to_one_level_and_one_post(pool: PgPool) {
        let author = create_guest(&pool, 1).await;
        let repo = CommentRepo::new(pool);
        let root = repo
            .create("post", author.id, None, "Root")
            .await
            .unwrap()
            .unwrap();
        let reply = repo
            .create("post", author.id, Some(root.id), "Reply")
            .await
            .unwrap()
            .unwrap();

        assert!(
            repo.create("post", author.id, Some(reply.id), "Nested")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            repo.create("other", author.id, Some(root.id), "Wrong post")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            repo.create("post", author.id, Some(CommentId(999)), "Missing")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[sqlx::test]
    async fn delete_owned_checks_the_author(pool: PgPool) {
        let owner = create_guest(&pool, 1).await;
        let other = create_guest(&pool, 2).await;
        let repo = CommentRepo::new(pool);
        let comment = repo
            .create("post", owner.id, None, "Mine")
            .await
            .unwrap()
            .unwrap();

        assert!(!repo.delete_owned(comment.id, other.id).await.unwrap());
        assert_eq!(repo.list("post").await.unwrap(), vec![comment.clone()]);
        assert!(repo.delete_owned(comment.id, owner.id).await.unwrap());
        assert!(repo.list("post").await.unwrap().is_empty());
    }
}

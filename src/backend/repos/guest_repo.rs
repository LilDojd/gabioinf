use crate::backend::errors::BResult;
use crate::shared::models::{GithubId, Guest, GuestId};

#[derive(Debug, Clone)]
pub struct GuestRepo {
    pool: sqlx::PgPool,
}

impl GuestRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, guest: &Guest) -> BResult<Guest> {
        Ok(sqlx::query_as!(
            Guest,
            r#"
            INSERT INTO guests (github_id, name, username)
            VALUES ($1, $2, $3)
            ON CONFLICT (github_id) DO UPDATE
            SET name = excluded.name, username = excluded.username, updated_at = NOW()
            RETURNING id AS "id: GuestId", github_id AS "github_id: GithubId", name, username, created_at, updated_at
            "#,
            guest.github_id.as_value(),
            guest.name,
            guest.username,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn find_by_id(&self, id: GuestId) -> BResult<Option<Guest>> {
        Ok(sqlx::query_as!(
            Guest,
            r#"SELECT id AS "id: GuestId", github_id AS "github_id: GithubId", name, username, created_at, updated_at FROM guests WHERE id = $1"#,
            id.as_value(),
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn find_by_username(&self, username: &str) -> BResult<Option<Guest>> {
        Ok(sqlx::query_as!(
            Guest,
            r#"SELECT id AS "id: GuestId", github_id AS "github_id: GithubId", name, username, created_at, updated_at FROM guests WHERE username = $1"#,
            username,
        )
        .fetch_optional(&self.pool)
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn upsert_and_fetch_guest(pool: PgPool) {
        let repo = GuestRepo::new(pool);
        let mut guest = Guest {
            github_id: GithubId(12345),
            name: "Test User".to_string(),
            username: "testuser".to_string(),
            ..Default::default()
        };

        let created = repo.upsert(&guest).await.unwrap();
        guest.name = "Updated User".to_string();
        let updated = repo.upsert(&guest).await.unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "Updated User");
        assert_eq!(
            repo.find_by_id(created.id).await.unwrap(),
            Some(updated.clone())
        );
        assert_eq!(
            repo.find_by_username("testuser").await.unwrap(),
            Some(updated)
        );
        assert_eq!(repo.find_by_username("missing").await.unwrap(), None);
    }
}

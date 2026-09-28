use crate::backend::errors::BResult;
use crate::shared::models::{GithubId, Guest, GuestId, NewGuest};

#[derive(Debug, Clone)]
pub struct GuestRepo {
    pool: sqlx::PgPool,
}

impl GuestRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, guest: &NewGuest) -> BResult<Guest> {
        Ok(sqlx::query_as!(
            Guest,
            r#"
            INSERT INTO guests (github_id, name, username)
            VALUES ($1, COALESCE($2, $3), $3)
            ON CONFLICT (github_id) DO UPDATE
            SET name = excluded.name, username = excluded.username, updated_at = NOW()
            RETURNING id AS "id: GuestId", github_id AS "github_id: GithubId", name, username, created_at, updated_at
            "#,
            guest.id.as_value(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn upsert_and_fetch_guest(pool: PgPool) {
        let repo = GuestRepo::new(pool);
        let mut guest = NewGuest {
            id: GithubId(12345),
            username: "testuser".to_string(),
            name: None,
        };

        let created = repo.upsert(&guest).await.unwrap();
        guest.name = Some("Updated User".to_string());
        let updated = repo.upsert(&guest).await.unwrap();

        assert_eq!(created.name, "testuser");
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "Updated User");
        assert_eq!(repo.find_by_id(created.id).await.unwrap(), Some(updated));
    }

    #[sqlx::test]
    async fn a_released_username_can_sign_in_on_another_account(pool: PgPool) {
        let repo = GuestRepo::new(pool);
        let guest = |id| NewGuest {
            id: GithubId(id),
            username: "shared".to_string(),
            name: None,
        };

        repo.upsert(&guest(1)).await.unwrap();

        assert_eq!(repo.upsert(&guest(2)).await.unwrap().github_id, GithubId(2));
    }
}

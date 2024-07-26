use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::{DateTime, Utc};

use crate::{
    db::DbConnPool,
    domain::models::{GithubId, GithubUser, Guest},
    errors::{BResult, BackendError, UseCase},
    AppState,
};

#[derive(Clone, Debug)]
pub struct GuestCrud {
    db: Arc<DbConnPool>,
}

impl GuestCrud {
    pub fn new(db: Arc<DbConnPool>) -> Self {
        Self { db }
    }

    pub async fn get_by_github_id(&self, github_id: &GithubId, usecase: UseCase) -> BResult<Guest> {
        sqlx::query_as::<_, Guest>("SELECT * FROM guests WHERE github_id = $1")
            .bind(github_id.as_value())
            .fetch_one(self.db.as_ref())
            .await
            .map_err(|err| BackendError::from((err, usecase)))
    }

    pub async fn get_by_id(&self, id: i64) -> BResult<Guest> {
        sqlx::query_as::<_, Guest>("SELECT * FROM guests WHERE id = $1")
            .bind(id)
            .fetch_one(self.db.as_ref())
            .await
            .map_err(BackendError::from)
    }

    pub async fn upsert_guest(&self, github_user: &GithubUser) -> BResult<Guest> {
        let guest = sqlx::query_as::<_, Guest>(
            "INSERT INTO guests (github_id, name, username) 
             VALUES ($1, $2, $3) 
             ON CONFLICT (github_id) DO UPDATE 
             SET name = EXCLUDED.name, username = EXCLUDED.username 
             RETURNING *",
        )
        .bind(github_user.id)
        .bind(&github_user.name)
        .bind(&github_user.login)
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(guest)
    }

    pub async fn promote_to_admin(&self, guest_id: i64) -> BResult<Guest> {
        let guest = sqlx::query_as::<_, Guest>(
            "UPDATE guests SET is_admin = true WHERE id = $1 RETURNING *",
        )
        .bind(guest_id)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(BackendError::from)?;

        Ok(guest)
    }

    pub async fn get_all_admins(&self) -> BResult<Vec<Guest>> {
        let admins = sqlx::query_as::<_, Guest>("SELECT * FROM guests WHERE is_admin = true")
            .fetch_all(self.db.as_ref())
            .await
            .map_err(BackendError::from)?;

        Ok(admins)
    }

    pub async fn register_session<S: AsRef<str>>(
        &self,
        guest: &Guest,
        token_secret: S,
        max_age: DateTime<Utc>,
    ) -> BResult<()> {
        sqlx::query(
            "INSERT INTO sessions (user_id, token, expires_at) 
             VALUES ($1, $2, $3) 
             ON CONFLICT (user_id) DO UPDATE 
             SET token = EXCLUDED.token, expires_at = EXCLUDED.expires_at",
        )
        .bind(guest.id)
        .bind(token_secret.as_ref())
        .bind(max_age)
        .execute(self.db.as_ref())
        .await
        .map_err(BackendError::from)?;

        Ok(())
    }

    pub async fn invalidate_session<S: AsRef<str>>(&self, token_secret: S) -> BResult<()> {
        sqlx::query("DELETE FROM sessions WHERE token = $1")
            .bind(token_secret.as_ref())
            .execute(self.db.as_ref())
            .await
            .map_err(BackendError::from)?;

        Ok(())
    }

    pub async fn get_session<S: AsRef<str>>(
        &self,
        token_secret: S,
        usecase: UseCase,
    ) -> BResult<(i64, DateTime<Utc>)> {
        let (user_id, expires_at) = sqlx::query_as::<_, (i64, DateTime<Utc>)>(
            "SELECT user_id, expires_at FROM sessions WHERE token = $1",
        )
        .bind(token_secret.as_ref())
        .fetch_one(self.db.as_ref())
        .await
        .map_err(|err| BackendError::from((err, usecase)))?;

        Ok((user_id, expires_at))
    }

    pub async fn get_guests(&self) -> BResult<Vec<Guest>> {
        sqlx::query_as::<_, Guest>("SELECT * FROM guests")
            .fetch_all(self.db.as_ref())
            .await
            .map_err(BackendError::from)
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Guest {
    type Rejection = BackendError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Guest>()
            .cloned()
            .ok_or(BackendError::Unauthorized)
    }
}

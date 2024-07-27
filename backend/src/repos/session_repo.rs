use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    db::DbConnPool,
    domain::models::{GuestId, Session, SessionId},
    errors::{ApiError, BResult},
};

use super::{PgRepository, Repository};

#[derive(Clone, Debug)]
pub struct SessionRepo {
    db: Arc<DbConnPool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionCriteria {
    WithId(SessionId),
    WithGuestId(GuestId),
    WithToken(String),
    Latest,
}

#[axum::async_trait]
impl Repository<Session> for PgRepository<Session> {
    type Error = ApiError;
    type Criteria = SessionCriteria;

    async fn read_all(&self) -> BResult<Vec<Session>> {
        let sessions = sqlx::query_as!(Session, "SELECT * FROM sessions ORDER BY issued_at")
            .fetch_all(&self.pool)
            .await?;
        Ok(sessions)
    }
    async fn read(&self, criteria: &Self::Criteria) -> BResult<Session> {
        let session = match criteria {
            SessionCriteria::WithId(id) => {
                sqlx::query_as!(
                    Session,
                    "SELECT * FROM sessions WHERE id = $1",
                    id.as_value()
                )
                .fetch_one(&self.pool)
                .await?
            }
            SessionCriteria::WithGuestId(guest_id) => {
                sqlx::query_as!(
                    Session,
                    "SELECT * FROM sessions WHERE user_id = $1",
                    guest_id.as_value()
                )
                .fetch_one(&self.pool)
                .await?
            }
            SessionCriteria::WithToken(token) => {
                sqlx::query_as!(Session, "SELECT * FROM sessions WHERE token = $1", token)
                    .fetch_one(&self.pool)
                    .await?
            }
            SessionCriteria::Latest => {
                sqlx::query_as!(Session, "SELECT * FROM sessions ORDER BY issued_at DESC")
                    .fetch_one(&self.pool)
                    .await?
            }
        };
        Ok(session)
    }

    async fn create(&self, session: &Session) -> BResult<Session> {
        let created_session = sqlx::query_as!(
            Session,
            "INSERT INTO sessions (user_id, token, expires_at) 
             VALUES ($1, $2, $3) 
             ON CONFLICT (user_id) DO UPDATE 
             SET token = EXCLUDED.token, expires_at = EXCLUDED.expires_at
             RETURNING *",
            session.user_id.as_value(),
            session.token,
            session.expires_at,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(created_session)
    }

    async fn update(&self, _session: &Session) -> BResult<Session> {
        Err(ApiError::NotImplementedErrpr(
            "Attempted to call update on sessions".to_string(),
        ))
    }

    async fn delete(&self, session: &Session) -> BResult<()> {
        sqlx::query!("DELETE FROM sessions WHERE id = $1", session.id.as_value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

impl PgRepository<Session> {}

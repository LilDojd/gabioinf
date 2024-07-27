use super::{PgRepository, Repository};
use crate::{
    domain::models::{GuestId, Session, SessionId},
    errors::{ApiError, BResult},
};
use serde::{Deserialize, Serialize};

/// Criteria for querying session data.
#[derive(Debug, Serialize, Deserialize)]
pub enum SessionCriteria {
    /// Query by session ID.
    WithId(SessionId),
    /// Query by guest ID.
    WithGuestId(GuestId),
    /// Query by session token.
    WithToken(String),
    /// Query for the latest session.
    Latest,
}

#[axum::async_trait]
impl Repository<Session> for PgRepository<Session> {
    type Error = ApiError;
    type Criteria = SessionCriteria;

    /// Retrieves all sessions from the database.
    async fn read_all(&self) -> BResult<Vec<Session>> {
        let sessions = sqlx::query_as!(Session, "SELECT * FROM sessions ORDER BY issued_at")
            .fetch_all(&self.pool)
            .await?;
        Ok(sessions)
    }

    /// Retrieves a single session based on the provided criteria.
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

    /// Creates a new session or updates an existing one if there's a conflict on user_id.
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

    /// Update operation is not implemented for sessions.
    async fn update(&self, _session: &Session) -> BResult<Session> {
        Err(ApiError::NotImplementedErrpr(
            "Attempted to call update on sessions".to_string(),
        ))
    }

    /// Deletes a session from the database.
    async fn delete(&self, session: &Session) -> BResult<()> {
        sqlx::query!("DELETE FROM sessions WHERE id = $1", session.id.as_value())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

impl PgRepository<Session> {}

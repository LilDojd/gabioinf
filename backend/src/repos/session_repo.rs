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

#[cfg(test)]
mod tests {
    use crate::utils::{setup_guest, setup_guests};

    use super::*;
    use chrono::{Duration, Utc};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_create_and_read_session(pool: PgPool) {
        setup_guest(&pool).await;

        let repo = PgRepository::<Session>::new(pool);
        let session = Session {
            user_id: GuestId(1),
            token: "test_token".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            ..Default::default()
        };

        // Test create
        let created_session = repo.create(&session).await.unwrap();
        assert_eq!(created_session.user_id, session.user_id);
        assert_eq!(created_session.token, session.token);

        // Test read by ID
        let read_session = repo
            .read(&SessionCriteria::WithId(created_session.id))
            .await
            .unwrap();
        assert_eq!(read_session.id, created_session.id);
        assert_eq!(read_session.token, session.token);

        // Test read by token
        let read_session = repo
            .read(&SessionCriteria::WithToken(session.token.clone()))
            .await
            .unwrap();
        assert_eq!(read_session.id, created_session.id);
        assert_eq!(read_session.user_id, session.user_id);
    }

    #[sqlx::test]
    async fn test_create_session_no_conflict(pool: PgPool) {
        let repo = PgRepository::<Session>::new(pool.clone());
        setup_guests(2, &pool).await;

        let session1 = Session {
            user_id: GuestId(1),
            token: "token1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            ..Default::default()
        };

        let session2 = Session {
            user_id: GuestId(2),
            token: "token2".to_string(),
            expires_at: Utc::now() + Duration::hours(2),
            ..Default::default()
        };

        // Create first session
        let created_session1 = repo.create(&session1).await.unwrap();

        // Create second session with same user_id (should update)
        let created_session2 = repo.create(&session2).await.unwrap();

        assert_ne!(created_session1.id, created_session2.id);
        assert_eq!(created_session2.token, "token2");
        assert_eq!(created_session2.user_id, GuestId(2));
    }

    #[sqlx::test]
    async fn test_delete_session(pool: PgPool) {
        setup_guest(&pool).await;

        let repo = PgRepository::<Session>::new(pool);
        let session = Session {
            user_id: GuestId(1),
            token: "delete_token".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            ..Default::default()
        };

        let created_session = repo.create(&session).await.unwrap();
        repo.delete(&created_session).await.unwrap();

        let result = repo
            .read(&SessionCriteria::WithId(created_session.id))
            .await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_update_session_not_implemented(pool: PgPool) {
        setup_guest(&pool).await;

        let repo = PgRepository::<Session>::new(pool);
        let session = Session {
            user_id: GuestId(1),
            token: "update_token".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            ..Default::default()
        };

        let result = repo.update(&session).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::NotImplementedErrpr(_)
        ));
    }

    #[sqlx::test]
    async fn test_read_all_sessions_single_user(pool: PgPool) {
        let repo = PgRepository::<Session>::new(pool.clone());

        setup_guest(&pool).await;

        // Create multiple sessions
        for i in 1..=3 {
            let session = Session {
                user_id: GuestId(1),
                token: format!("token{}", i),
                expires_at: Utc::now() + Duration::hours(1),
                ..Default::default()
            };
            repo.create(&session).await.unwrap();
        }

        let all_sessions = repo.read_all().await.unwrap();
        assert!(all_sessions.len() == 3);
    }

    #[sqlx::test]
    async fn test_read_all_sessions_multiple_users(pool: PgPool) {
        let repo = PgRepository::<Session>::new(pool.clone());

        setup_guests(3, &pool).await;

        // Create multiple sessions
        for i in 1..=3 {
            let session = Session {
                user_id: GuestId(i),
                token: format!("token{}", i),
                expires_at: Utc::now() + Duration::hours(1),
                ..Default::default()
            };
            repo.create(&session).await.unwrap();
        }

        let all_sessions = repo.read_all().await.unwrap();
        assert!(all_sessions.len() == 3);
    }

    #[sqlx::test]
    #[should_panic]
    async fn create_session_without_guest(pool: PgPool) {
        let repo = PgRepository::<Session>::new(pool);
        let session = Session {
            user_id: GuestId(1),
            token: "test_token".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            ..Default::default()
        };

        repo.create(&session).await.unwrap();
    }
}

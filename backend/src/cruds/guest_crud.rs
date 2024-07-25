use std::sync::Arc;

use axum::extract::{FromRequest, FromRequestParts, Request};
use axum_extra::extract::PrivateCookieJar;
use chrono::NaiveDateTime;

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

    // More to follow..
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

    pub async fn register_session(
        &self,
        guest: &Guest,
        token_secret: &String,
        max_age: NaiveDateTime,
    ) -> BResult<()> {
        match sqlx::query(
            "INSERT INTO sessions (user_id, session_id, expires_at) 
             VALUES ($1, $2, $3) 
             ON CONFLICT (user_id) DO UPDATE 
             SET session_id = EXCLUDED.session_id, expires_at = EXCLUDED.expires_at",
        )
        .bind(guest.id)
        .bind(token_secret)
        .bind(max_age)
        .execute(self.db.as_ref())
        .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(BackendError::from(err)),
        }
    }

    pub async fn get_guests(&self) -> BResult<Vec<Guest>> {
        sqlx::query_as::<_, Guest>("SELECT * FROM guests")
            .fetch_all(self.db.as_ref())
            .await
            .map_err(BackendError::from)
    }
}

#[axum::async_trait]
impl FromRequest<AppState> for Guest {
    type Rejection = BackendError;
    async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();
        let (mut parts, _body) = req.into_parts();
        let cookiejar: PrivateCookieJar =
            PrivateCookieJar::from_request_parts(&mut parts, &state).await?;

        let Some(cookie) = cookiejar.get("sid").map(|cookie| cookie.value().to_owned()) else {
            tracing::debug!("No session cookie found. Cookiejar state: {:?}", cookiejar);
            return Err(BackendError::Unauthorized);
        };

        let res = sqlx::query_as::<_, Guest>(
            "SELECT * FROM sessions
            LEFT JOIN GUESTS ON sessions.user_id = guests.id
            WHERE sessions.session_id = $1
            LIMIT 1",
        )
        .bind(cookie)
        .fetch_one(state.db.as_ref())
        .await?;

        Ok(res)
    }
}

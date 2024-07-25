use std::sync::Arc;

use crate::{
    db::DbConnPool,
    domain::models::{GithubId, Guest},
    errors::{BResult, BackendError, UseCase},
};

#[derive(Clone, Debug)]
struct GuestCrud {
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
}

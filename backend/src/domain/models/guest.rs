use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Default, Clone)]
pub struct GithubId(i64);

impl GithubId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

impl From<i64> for GithubId {
    fn from(id: i64) -> Self {
        GithubId(id)
    }
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Guest {
    pub id: i64,
    pub github_id: String,
    pub name: String,
    pub username: String,
    pub is_naughty: bool,
    pub naughty_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

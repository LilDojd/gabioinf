use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Database, Decode, FromRow};

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
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

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct GuestId(i64);

impl GuestId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

impl From<i64> for GuestId {
    fn from(id: i64) -> Self {
        GuestId(id)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Guest {
    pub id: GuestId,
    pub github_id: GithubId,
    pub name: String,
    pub username: String,
    pub is_naughty: bool,
    pub is_admin: bool,
    pub naughty_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct GithubUser {
    pub id: GithubId,
    pub login: String,
    pub name: Option<String>,
}

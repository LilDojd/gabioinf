use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

use super::Guest;

#[derive(Debug, Serialize, FromRow)]
pub struct GuestbookEntry {
    #[serde(skip_serializing)]
    pub id: i64,
    pub message: String,
    pub signature: Option<String>, // Base64 encoded image data
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: Guest,
}

#[derive(Debug)]
pub struct NewGuestbookEntry {
    pub name: String,
    pub message: String,
    pub signature: Option<String>,
}

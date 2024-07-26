use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::GuestId;

#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct GuestbookEntry {
    #[serde(skip_serializing)]
    pub id: i64,
    pub message: String,
    pub signature: Option<String>, // Base64 encoded image data
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_id: GuestId,
}

#[derive(Debug)]
pub struct NewGuestbookEntry {
    pub name: String,
    pub message: String,
    pub signature: Option<String>,
}

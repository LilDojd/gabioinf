use super::GuestId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents an entry in the guestbook.
#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct GuestbookEntry {
    /// The unique identifier for the guestbook entry.
    /// This field is not serialized when the struct is converted to JSON.
    #[serde(skip_serializing)]
    pub id: i64,

    /// The message content of the guestbook entry.
    pub message: String,

    /// An optional signature for the guestbook entry.
    /// This is typically stored as Base64 encoded image data.
    pub signature: Option<String>,

    /// The timestamp when the guestbook entry was created.
    pub created_at: DateTime<Utc>,

    /// The timestamp when the guestbook entry was last updated.
    pub updated_at: DateTime<Utc>,

    /// The ID of the guest who authored this entry.
    pub author_id: GuestId,
}

/// Represents the data required to create a new guestbook entry.
#[derive(Debug)]
pub struct NewGuestbookEntry {
    /// The name of the guest creating the entry.
    pub name: String,

    /// The message content for the new guestbook entry.
    pub message: String,

    /// An optional signature for the new guestbook entry.
    /// This is typically provided as Base64 encoded image data.
    pub signature: Option<String>,
}

use super::GuestId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
extern crate derive_more;
use derive_more::{From, Into};

/// Represents an ID of a guestbook entry
///
/// This type is a newtype wrapper around `i64` to provide type safety and clarity
/// when dealing with guestbook IDs.
#[derive(
    Debug, Serialize, Deserialize, Default, Clone, Copy, sqlx::Type, From, Into, PartialEq,
)]
#[sqlx(transparent)]
pub struct GuestbookId(pub(crate) i64);

impl GuestbookId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

/// Represents an entry in the guestbook.
#[derive(Debug, Serialize, FromRow, Deserialize, Clone, Default)]
pub struct GuestbookEntry {
    /// The unique identifier for the guestbook entry.
    /// This field is not serialized when the struct is converted to JSON.
    pub id: GuestbookId,

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
    /// The id of the guest creating the entry.
    pub author_id: GuestId,

    /// The message content for the new guestbook entry.
    pub message: String,

    /// An optional signature for the new guestbook entry.
    /// This is typically provided as Base64 encoded image data.
    pub signature: Option<String>,
}

impl From<NewGuestbookEntry> for GuestbookEntry {
    fn from(entry: NewGuestbookEntry) -> Self {
        Self {
            message: entry.message,
            signature: entry.signature,
            author_id: entry.author_id,
            ..Default::default()
        }
    }
}

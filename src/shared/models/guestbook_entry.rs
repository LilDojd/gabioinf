use super::GuestId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::{FromRow, Type};
use time::OffsetDateTime;
extern crate derive_more;
use derive_more::{From, Into};
/// Represents an ID of a guestbook entry
///
/// This type is a newtype wrapper around `i64` to provide type safety and clarity
/// when dealing with guestbook IDs.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, From, Into, PartialEq)]
#[cfg_attr(feature = "server", derive(Type), sqlx(transparent))]
pub struct GuestbookId(pub(crate) i64);
impl GuestbookId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}
/// Represents an entry in the guestbook.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "server", derive(FromRow), sqlx(transparent))]
pub struct GuestbookEntry {
    /// The unique identifier for the guestbook entry.
    pub id: GuestbookId,
    /// The message content of the guestbook entry.
    pub message: String,
    /// An optional signature for the guestbook entry.
    /// This is typically stored as Base64 encoded image data.
    pub signature: Option<String>,
    /// The timestamp when the guestbook entry was created.
    pub created_at: OffsetDateTime,
    /// The timestamp when the guestbook entry was last updated.
    pub updated_at: OffsetDateTime,
    /// The ID of the guest who authored this entry.
    pub author_id: GuestId,
    /// The username of the guest who authored this entry.
    pub author_username: String,
}
impl Default for GuestbookEntry {
    fn default() -> Self {
        Self {
            id: GuestbookId(0),
            message: "".to_string(),
            signature: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            author_id: GuestId(0),
            author_username: "".to_string(),
        }
    }
}
/// Represents the data required to create a new guestbook entry.
#[derive(Debug)]
pub struct NewGuestbookEntry {
    /// The id of the guest creating the entry.
    pub author_id: GuestId,
    /// The username of the guest creating the entry.
    pub author_username: String,
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
            author_username: entry.author_username,
            ..Default::default()
        }
    }
}

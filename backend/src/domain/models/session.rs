use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
extern crate derive_more;
use derive_more::{From, Into};

use super::GuestId;

/// Represents a session ID in the system.
///
/// This type is a newtype wrapper around `i64` to provide type safety and clarity
/// when dealing with session IDs.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, sqlx::Type, From, Into)]
#[sqlx(transparent)]
pub struct SessionId(i64);

impl SessionId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

/// Represents a session in the system.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, Default)]
pub struct Session {
    /// The unique identifier for the session.
    pub id: SessionId,
    /// The ID of the user associated with the session.
    pub user_id: GuestId,
    /// The session token.
    pub token: String,
    /// The timestamp when the session was issued.
    pub issued_at: DateTime<Utc>,
    /// The timestamp when the session expires.
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new<S: AsRef<str>>(user_id: GuestId, token: S, expires_at: DateTime<Utc>) -> Self {
        let token = token.as_ref().to_string();
        Self {
            user_id,
            token,
            expires_at,
            ..Default::default()
        }
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a GitHub user ID.
///
/// This type is a wrapper around `i64` to provide type safety and clarity
/// when dealing with GitHub user IDs.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct GithubId(i64);

impl GithubId {
    /// Returns the inner `i64` value of the `GithubId`.
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

impl From<i64> for GithubId {
    /// Creates a `GithubId` from an `i64`.
    fn from(id: i64) -> Self {
        GithubId(id)
    }
}

/// Represents a guest ID in the system.
///
/// This type is a wrapper around `i64` to provide type safety and clarity
/// when dealing with guest IDs.
#[derive(
    Debug, Serialize, Deserialize, Default, Clone, Copy, sqlx::Type, PartialEq, Eq, PartialOrd, Ord,
)]
#[sqlx(transparent)]
pub struct GuestId(i64);

impl GuestId {
    /// Returns the inner `i64` value of the `GuestId`.
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

impl From<i64> for GuestId {
    /// Creates a `GuestId` from an `i64`.
    fn from(id: i64) -> Self {
        GuestId(id)
    }
}

/// Represents a guest in the system.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Guest {
    /// The unique identifier for the guest.
    pub id: GuestId,
    /// The GitHub ID associated with the guest.
    pub github_id: GithubId,
    /// The full name of the guest.
    pub name: String,
    /// The username of the guest.
    pub username: String,
    /// Indicates whether the guest is marked as naughty.
    pub is_naughty: bool,
    /// Indicates whether the guest has admin privileges.
    pub is_admin: bool,
    /// The reason for marking the guest as naughty, if applicable.
    pub naughty_reason: Option<String>,
    /// The timestamp when the guest record was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp when the guest record was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Represents a GitHub user as returned by the GitHub API.
#[derive(Deserialize)]
pub struct GithubUser {
    /// The GitHub user ID.
    pub id: GithubId,
    /// The GitHub username (login).
    pub login: String,
    /// The full name of the GitHub user, if available.
    pub name: Option<String>,
}

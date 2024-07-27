use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
extern crate derive_more;
use derive_more::{From, Into};

/// Represents a GitHub user ID.
///
/// This type is a newtype wrapper around `i64` to provide type safety and clarity
/// when dealing with GitHub user IDs.
// TODO: Strip
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Default,
    Clone,
    Copy,
    sqlx::Type,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Into,
)]
#[sqlx(transparent)]
pub struct GithubId(i64);

impl GithubId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

/// Represents a guest ID in the system.
///
/// This type is a wrapper around `i64` to provide type safety and clarity
/// when dealing with guest IDs.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Default,
    Clone,
    Copy,
    sqlx::Type,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Into,
)]
#[sqlx(transparent)]
pub struct GuestId(i64);

impl GuestId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

/// Represents a guest in the system.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, Default)]
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
#[derive(Deserialize, Debug)]
pub struct NewGuest {
    /// The GitHub user ID.
    pub id: GithubId,
    /// The GitHub username (login).
    #[serde(rename(deserialize = "login"))]
    pub username: String,
    /// The full name of the GitHub user, if available.
    pub name: Option<String>,
}

impl From<NewGuest> for Guest {
    fn from(val: NewGuest) -> Self {
        Guest {
            github_id: val.id,
            name: val.name.unwrap_or_else(|| val.username.clone()),
            username: val.username,
            ..Default::default()
        }
    }
}

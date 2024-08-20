use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::{FromRow, Type};
use time::OffsetDateTime;
extern crate derive_more;
use derive_more::{From, Into};
/// Represents a GitHub user ID.
///
/// This type is a newtype wrapper around `i64` to provide type safety and clarity
/// when dealing with GitHub user IDs.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Into,
)]
pub struct GithubId(pub(crate) i64);
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
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Into,
)]
#[cfg_attr(feature = "server", derive(Type), sqlx(transparent))]
pub struct GuestId(pub(crate) i64);
impl std::fmt::Display for GuestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl GuestId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}
/// Represents a guest in the system.
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "server", derive(FromRow), sqlx(transparent))]
pub struct Guest {
    /// The unique identifier for the guest.
    pub id: GuestId,
    /// The GitHub ID associated with the guest.
    pub github_id: GithubId,
    /// The full name of the guest.
    pub name: String,
    /// The username of the guest.
    pub username: String,
    /// The timestamp when the guest record was created.
    pub created_at: OffsetDateTime,
    /// The timestamp when the guest record was last updated.
    pub updated_at: OffsetDateTime,
    /// Access token for the guest.
    pub access_token: String,
}
impl Default for Guest {
    fn default() -> Self {
        Self {
            id: GuestId(0),
            github_id: GithubId(0),
            name: "".to_string(),
            username: "".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            access_token: "".to_string(),
        }
    }
}
impl std::fmt::Debug for Guest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Guest")
            .field("id", &self.id)
            .field("github_id", &self.github_id)
            .field("name", &self.name)
            .field("username", &self.username)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .field("access_token", &"********")
            .finish()
    }
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

use super::GithubId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use sqlx::Type;
use time::OffsetDateTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(Type), sqlx(transparent))]
pub struct CommentId(pub(crate) i64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommentAuthor {
    pub username: String,
    pub github_id: GithubId,
    pub is_owner: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: CommentId,
    pub parent_id: Option<CommentId>,
    pub author: CommentAuthor,
    pub body_html: String,
    pub created_at: OffsetDateTime,
}

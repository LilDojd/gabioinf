mod delete_comment;
pub use delete_comment::*;
mod load_comments;
pub use load_comments::*;
mod post_comment;
pub use post_comment::*;

#[cfg(feature = "server")]
use crate::{
    backend::{markdown, repos::CommentRow},
    shared::models::{Comment, CommentAuthor, OWNER_GITHUB_ID},
};

#[cfg(feature = "server")]
fn render_comment(row: CommentRow) -> Result<Comment, markdown::MarkdownError> {
    let body_html = markdown::render(&row.body)?;
    Ok(comment_with_html(row, body_html))
}

#[cfg(feature = "server")]
fn comment_with_html(row: CommentRow, body_html: String) -> Comment {
    Comment {
        id: row.id,
        parent_id: row.parent_id,
        author: CommentAuthor {
            username: row.username,
            github_id: row.github_id,
            is_owner: row.github_id == OWNER_GITHUB_ID,
        },
        body_html,
        created_at: row.created_at,
    }
}

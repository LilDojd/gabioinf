use crate::backend::repos::{CommentRepo, GuestRepo, GuestbookRepo, ReactionRepo};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub guest_repo: GuestRepo,
    pub guestbook_repo: GuestbookRepo,
    pub comment_repo: CommentRepo,
    pub reaction_repo: ReactionRepo,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            guest_repo: GuestRepo::new(db.clone()),
            guestbook_repo: GuestbookRepo::new(db.clone()),
            comment_repo: CommentRepo::new(db.clone()),
            reaction_repo: ReactionRepo::new(db),
        }
    }
}

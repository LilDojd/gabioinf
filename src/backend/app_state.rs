use crate::backend::{
    db::DbConnPool,
    repos::{CommentRepo, GuestRepo, GuestbookRepo, ReactionRepo},
};
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: DbConnPool,
    pub guest_repo: GuestRepo,
    pub guestbook_repo: GuestbookRepo,
    pub comment_repo: CommentRepo,
    pub reaction_repo: ReactionRepo,
    /// `https://gabioinf.dev` in production, `http://localhost:8080` locally.
    pub origin: String,
    pub key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl AppState {
    pub fn new(db: DbConnPool, origin: String, key: Key) -> Self {
        Self {
            db: db.clone(),
            guest_repo: GuestRepo::new(db.clone()),
            guestbook_repo: GuestbookRepo::new(db.clone()),
            comment_repo: CommentRepo::new(db.clone()),
            reaction_repo: ReactionRepo::new(db),
            origin,
            key,
        }
    }
}

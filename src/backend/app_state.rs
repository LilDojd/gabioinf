use crate::backend::{
    db::DbConnPool,
    repos::{CommentRepo, GuestRepo, GuestbookRepo},
};
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: DbConnPool,
    pub guest_repo: GuestRepo,
    pub guestbook_repo: GuestbookRepo,
    pub comment_repo: CommentRepo,
    pub domain: String,
    pub key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl AppState {
    pub fn new(db: DbConnPool, domain: String, key: Key) -> Self {
        Self {
            db: db.clone(),
            guest_repo: GuestRepo::new(db.clone()),
            guestbook_repo: GuestbookRepo::new(db.clone()),
            comment_repo: CommentRepo::new(db),
            domain,
            key,
        }
    }
}

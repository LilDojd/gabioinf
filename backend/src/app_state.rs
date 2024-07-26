use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use reqwest::Client as ReqwestClient;

use crate::{
    cruds::{GuestCrud, GuestbookCrud},
    db::DbConnPool,
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: Arc<DbConnPool>,
    pub ctx: ReqwestClient,
    pub guest_crud: Arc<GuestCrud>,
    pub guestbook_crud: Arc<GuestbookCrud>,
    pub domain: String,
    pub key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl AppState {
    pub fn new(db: DbConnPool, domain: String) -> Self {
        let db = Arc::new(db);
        Self {
            db: db.clone(),
            ctx: ReqwestClient::new(),
            guest_crud: Arc::new(GuestCrud::new(db.clone())),
            guestbook_crud: Arc::new(GuestbookCrud::new(db)),
            domain,
            key: Key::generate(),
        }
    }
}

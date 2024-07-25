use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;

use crate::db::DbConnPool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: Arc<DbConnPool>,
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
        Self {
            db: Arc::new(db),
            domain,
            key: Key::generate(),
        }
    }
}

use std::sync::Arc;

use crate::{db::DbConnPool, domain::models::GuestbookEntry, errors::BResult};

#[derive(Clone, Debug)]
pub struct GuestbookCrud {
    db: Arc<DbConnPool>,
}

impl GuestbookCrud {
    pub fn new(db: Arc<DbConnPool>) -> Self {
        Self { db }
    }
}

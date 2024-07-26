use std::sync::Arc;

use chrono::Utc;

use crate::{
    db::DbConnPool,
    domain::models::{GuestId, GuestbookEntry},
    errors::{BResult, BackendError},
};

#[derive(Clone, Debug)]
pub struct GuestbookCrud {
    db: Arc<DbConnPool>,
}

impl GuestbookCrud {
    pub fn new(db: Arc<DbConnPool>) -> Self {
        Self { db }
    }

    pub async fn create_entry<S: AsRef<str>>(
        &self,
        guest_id: &GuestId,
        message: S,
    ) -> BResult<GuestbookEntry> {
        let entry = sqlx::query_as::<_, GuestbookEntry>(
            "INSERT INTO guestbook (message, author_id) VALUES ($1, $2) RETURNING *",
        )
        .bind(message.as_ref())
        .bind(guest_id)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(BackendError::from)?;

        Ok(entry)
    }
    //
    pub async fn get_all_entries(&self) -> BResult<Vec<GuestbookEntry>> {
        let entries =
            sqlx::query_as::<_, GuestbookEntry>("SELECT * FROM guestbook ORDER BY created_at DESC")
                .fetch_all(self.db.as_ref())
                .await
                .map_err(BackendError::from)?;

        Ok(entries)
    }

    pub async fn update_entry(&self, id: i64, message: &str) -> BResult<GuestbookEntry> {
        let entry = sqlx::query_as::<_, GuestbookEntry>(
            "UPDATE guestbook SET message = $1, updated_at = $2 WHERE id = $3 RETURNING *",
        )
        .bind(message)
        .bind(Utc::now())
        .bind(id)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(BackendError::from)?;

        Ok(entry)
    }

    pub async fn delete_entry(&self, id: i64) -> BResult<()> {
        sqlx::query("DELETE FROM guestbook WHERE id = $1")
            .bind(id)
            .execute(self.db.as_ref())
            .await
            .map_err(BackendError::from)?;

        Ok(())
    }

    pub async fn flag_as_naughty(&self, id: i64, reason: &str) -> BResult<GuestbookEntry> {
        let entry = sqlx::query_as::<_, GuestbookEntry>(
            "UPDATE guestbook SET is_naughty = true, naughty_reason = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(reason)
        .bind(Utc::now())
        .bind(id)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(BackendError::from)?;

        Ok(entry)
    }

    pub async fn get_naughty_entries(&self) -> BResult<Vec<GuestbookEntry>> {
        let entries = sqlx::query_as::<_, GuestbookEntry>(
            "SELECT * FROM guestbook WHERE is_naughty = true ORDER BY created_at DESC",
        )
        .fetch_all(self.db.as_ref())
        .await
        .map_err(BackendError::from)?;

        Ok(entries)
    }
}

//! Application state and shared resources.
//!
//! This module defines the `AppState` struct, which holds shared resources and
//! configuration for the application. It's designed to be shared across
//! different parts of the application, particularly in request handlers.
use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use reqwest::Client as ReqwestClient;

use crate::{
    db::DbConnPool,
    repos::{GuestRepo, GuestbookRepo},
};

/// Represents the shared state of the application.
///
/// This struct holds various shared resources and configuration that can be
/// accessed throughout the application, particularly in request handlers.
#[derive(Clone, Debug)]
pub struct AppState {
    /// The database connection pool, wrapped in an Arc for thread-safe sharing.
    pub db: Arc<DbConnPool>,
    /// An HTTP client for making external requests.
    pub ctx: ReqwestClient,
    /// Repository for guest-related data.
    pub guest_repo: Arc<GuestRepo>,
    /// Repository for guestbook-related data.
    pub guestbook_repo: Arc<GuestbookRepo>,
    /// The domain name of the application.
    pub domain: String,
    /// A key used for signing and verifying cookies.
    pub key: Key,
}

/// Allows extracting the `Key` from `AppState`.
///
/// This implementation is used by the Axum framework to extract the `Key`
/// when it's needed in request handlers or middleware.
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl AppState {
    /// Creates a new instance of `AppState`.
    ///
    /// This method initializes all the shared resources and wraps them
    /// in the appropriate smart pointers for thread-safe sharing.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection pool.
    /// * `domain` - The domain name of the application.
    ///
    /// # Returns
    ///
    /// A new instance of `AppState`.
    pub fn new(db: DbConnPool, domain: String) -> Self {
        let db = Arc::new(db);
        Self {
            db: db.clone(),
            ctx: ReqwestClient::new(),
            guest_repo: Arc::new(GuestRepo::new(db.clone())),
            guestbook_repo: Arc::new(GuestbookRepo::new(db)),
            domain,
            key: Key::generate(),
        }
    }
}

//! Application state and shared resources.
//!
//! This module defines the `AppState` struct, which holds shared resources and
//! configuration for the application. It's designed to be shared across
//! different parts of the application, particularly in request handlers.

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use reqwest::Client as ReqwestClient;

use crate::{
    db::DbConnPool,
    domain::models::{Guest, GuestbookEntry, Session},
    repos::PgRepository,
};

/// Represents the shared state of the application.
///
/// This struct holds various shared resources and configuration that can be
/// accessed throughout the application, particularly in request handlers.
#[derive(Clone, Debug)]
pub struct AppState {
    /// The database connection pool.
    pub db: DbConnPool,
    /// An HTTP client for making external requests.
    pub ctx: ReqwestClient,
    /// Repository for guest-related data.
    pub guest_repo: PgRepository<Guest>,
    /// Repository for guestbook-related data.
    pub guestbook_repo: PgRepository<GuestbookEntry>,
    /// Repository for user sessions
    pub session_repo: PgRepository<Session>,
    /// The domain name of the application.
    pub domain: String,
    /// A key used for signing and verifying cookies.
    pub key: Key,
    /// The client secret for the OAuth2 client.
    pub client_secret: String,
    /// The client ID for the OAuth2 client.
    pub client_id: String,
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
    /// * `client_secret` - The client secret for the OAuth2 client.
    /// * `client_id` - The client ID for the OAuth2 client.
    ///
    /// # Returns
    ///
    /// A new instance of `AppState`.
    pub fn new(db: DbConnPool, domain: String, client_secret: String, client_id: String) -> Self {
        Self {
            db: db.clone(),
            ctx: ReqwestClient::new(),
            guest_repo: PgRepository::new(db.clone()),
            guestbook_repo: PgRepository::new(db.clone()),
            session_repo: PgRepository::new(db.clone()),
            domain,
            client_secret,
            client_id,
            key: Key::generate(),
        }
    }
}

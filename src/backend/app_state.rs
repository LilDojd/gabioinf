//! Application state and shared resources.
//!
//! This module defines the `AppState` struct, which holds shared resources and
//! configuration for the application. It's designed to be shared across
//! different parts of the application, particularly in request handlers.
use crate::{
    backend::{db::DbConnPool, repos::{GroupsAndPermissionsRepo, PgRepository}},
    shared::models::{Guest, GuestbookEntry},
};
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use oauth2::basic::BasicClient;
use reqwest::Client as ReqwestClient;
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
    /// Repository for user and group permissions
    pub gp_repo: GroupsAndPermissionsRepo,
    /// The domain name of the application.
    pub domain: String,
    /// A key used for signing and verifying cookies.
    pub key: Key,
    /// The client for OAuth2 requests.
    pub client: BasicClient,
}
/// Allows extracting the `Key` from `AppState`.
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
    /// * `client` - The client for the OAuth2 requests.
    ///
    /// # Returns
    ///
    /// A new instance of `AppState`.
    pub fn new(db: DbConnPool, domain: String, client: BasicClient) -> Self {
        Self {
            db: db.clone(),
            ctx: ReqwestClient::new(),
            guest_repo: PgRepository::new(db.clone()),
            guestbook_repo: PgRepository::new(db.clone()),
            gp_repo: GroupsAndPermissionsRepo::new(db.clone()),
            domain,
            client,
            key: Key::generate(),
        }
    }
}

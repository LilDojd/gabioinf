//! Repositories for the backend.
//!
//! Repositories are responsible for handling the database operations according to the repository
//! pattern and are aimed to provide an abstraction layer between the database and the rest of the
//! application. This allows for easier testing and swapping out the database implementation in the
//! future.

/// Defines trait that provides interface for repositories.
mod repository;
pub use repository::*;

/// Guestbook repository.
mod guestbook_repo;
pub use guestbook_repo::*;

/// Guest repository.
mod guest_repo;
pub use guest_repo::*;

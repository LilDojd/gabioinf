//! # Backend Crate
//!
//! This crate provides the backend functionality for a web application,
//! including database interactions, API endpoints, and business logic.
//!
//! ## Modules
//!
//! - `app_state`: Manages the application's state and shared resources.
//! - `db`: Handles database connections and operations.
//! - `wapi`: Implements web API endpoints and request handling.
//! - `cruds`: Provides CRUD (Create, Read, Update, Delete) operations for data models.
//! - `domain`: Defines core domain models and business logic.
//! - `errors`: Centralizes error handling and custom error types.
//! - `shuttle_utils`: Utilities for working with the Shuttle deployment platform.
//! - `config`: Manages application configuration and environment-specific settings.

/// Re-exports all items from the `app_state` module
mod app_state;
pub use app_state::*;

/// Database-related functionality
pub mod db;

/// Web API implementation
pub mod wapi;

/// CRUD operations for data models
pub mod repos;

/// Core domain models and business logic
pub mod domain;

/// Error handling and custom error types
pub mod errors;

/// Utilities
pub mod utils;

/// Application configuration management
pub mod config;

/// Extractors for rate limiting and other middleware
pub mod extractors;

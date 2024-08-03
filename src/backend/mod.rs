//! # Backend
//!
//! This module provides the backend functionality for a web application,
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
/// Application configuration management
pub mod config;
/// Database-related functionality
pub mod db;
/// Core domain models and business logic
pub mod domain;
/// Error handling and custom error types
pub mod errors;
/// Extractors for rate limiting and other middleware
pub mod extractors;
/// CRUD operations for data models
pub mod repos;
/// Utilities
pub mod utils;
// pub mod wapi;

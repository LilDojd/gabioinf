//! Guestbook entry retrieval handlers.
//!
//! This module contains handler functions for retrieving guestbook entries,
//! including all entries and specifically naughty entries.

use crate::{errors::BResult, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// Handler for retrieving all guestbook entries.
///
/// This function fetches all guestbook entries from the database.
///
/// # Arguments
///
/// * `state` - The application state, containing the guestbook CRUD operations.
///
/// # Returns
///
/// Returns a `BResult` containing a JSON array of all guestbook entries if successful,
/// or an error if the retrieval fails.
///
/// # Response
///
/// * `200 OK` with a JSON array of guestbook entries.
///
/// # Errors
///
/// This function will return an error if the database query fails.
pub async fn get_all_entries(State(state): State<AppState>) -> BResult<impl IntoResponse> {
    tracing::debug!("Retrieving all guestbook entries");
    let entries = state.guestbook_repo.get_all_entries().await?;
    Ok((StatusCode::OK, Json(entries)))
}

/// Handler for retrieving all naughty guestbook entries.
///
/// This function fetches all guestbook entries that have been flagged as naughty.
///
/// # Arguments
///
/// * `state` - The application state, containing the guestbook CRUD operations.
///
/// # Returns
///
/// Returns a `BResult` containing a JSON array of all naughty guestbook entries if successful,
/// or an error if the retrieval fails.
///
/// # Response
///
/// * `200 OK` with a JSON array of naughty guestbook entries.
///
/// # Errors
///
/// This function will return an error if the database query fails.
pub async fn get_naughty_entries(State(state): State<AppState>) -> BResult<impl IntoResponse> {
    tracing::debug!("Retrieving all naughty guestbook entries");
    let entries = state.guestbook_repo.get_naughty_entries().await?;
    Ok((StatusCode::OK, Json(entries)))
}

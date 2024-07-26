//! Guestbook entry management handlers.
//!
//! This module contains handler functions for updating guestbook entries
//! and flagging entries as naughty.

use crate::{errors::BResult, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

/// Request payload for updating a guestbook entry.
#[derive(Deserialize, Debug)]
pub struct UpdateEntryRequest {
    /// The new message for the guestbook entry.
    message: String,
}

/// Request payload for flagging a guestbook entry as naughty.
#[derive(Deserialize)]
pub struct FlagNaughtyRequest {
    /// The reason for flagging the entry as naughty.
    reason: String,
}

/// Handler for updating a guestbook entry.
///
/// This function updates the message of a guestbook entry identified by its ID.
///
/// # Arguments
///
/// * `state` - The application state, containing the guestbook CRUD operations.
/// * `id` - The ID of the guestbook entry to update.
/// * `payload` - The new message for the guestbook entry.
///
/// # Returns
///
/// Returns a `BResult` containing the updated entry if successful, or an error if the update fails.
pub async fn update_entry(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateEntryRequest>,
) -> BResult<impl IntoResponse> {
    tracing::debug!("Updating guestbook entry with ID: {}", id);
    let entry = state
        .guestbook_crud
        .update_entry(id, &payload.message)
        .await?;
    Ok((StatusCode::OK, Json(entry)))
}

/// Handler for flagging a guestbook entry as naughty.
///
/// This function marks a guestbook entry as naughty and records the reason for doing so.
///
/// # Arguments
///
/// * `state` - The application state, containing the guestbook CRUD operations.
/// * `id` - The ID of the guestbook entry to flag.
/// * `payload` - The reason for flagging the entry as naughty.
///
/// # Returns
///
/// Returns a `BResult` containing the flagged entry if successful, or an error if the operation fails.
pub async fn flag_as_naughty(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<FlagNaughtyRequest>,
) -> BResult<impl IntoResponse> {
    tracing::debug!("Flagging guestbook entry with ID: {} as naughty", id);
    let entry = state
        .guestbook_crud
        .flag_as_naughty(id, &payload.reason)
        .await?;
    Ok((StatusCode::OK, Json(entry)))
}

//! Guest retrieval handler.
//!
//! This module contains the handler function for retrieving all guests.

use crate::{errors::BResult, AppState};
use axum::{extract::State, response::IntoResponse, Json};

/// Handler for retrieving all guests.
///
/// This function fetches all guests from the database.
///
/// # Arguments
///
/// * `state` - The application state, containing the guest CRUD operations.
///
/// # Returns
///
/// Returns a `BResult` containing a JSON array of all guests if successful,
/// or an error if the retrieval fails.
///
/// # Response
///
/// * `200 OK` with a JSON array of guests.
///
/// # Errors
///
/// This function will return an error if the database query fails.
///
/// # Security Considerations
///
/// - This endpoint should be protected and only accessible to authorized users,
///   preferably administrators.
/// - Consider implementing pagination if the number of guests can be large.
///
/// # Example
///
/// ```json
/// GET /guests/all
/// Authorization: Bearer <token>
/// ```
///
/// Successful response:
/// ```json
/// HTTP/1.1 200 OK
/// Content-Type: application/json
///
/// [
///   {
///     "id": 1,
///     "name": "John Doe",
///     "email": "john@example.com"
///   },
///   {
///     "id": 2,
///     "name": "Jane Smith",
///     "email": "jane@example.com"
///   }
/// ]
/// ```
pub async fn get_guests(State(state): State<AppState>) -> BResult<impl IntoResponse> {
    tracing::debug!("Retrieving all guests");
    let guests = state.guest_repo.get_guests().await?;
    Ok((axum::http::StatusCode::OK, Json(guests)))
}

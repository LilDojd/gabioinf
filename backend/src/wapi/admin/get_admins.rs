//! Admin retrieval handler.
//!
//! This module contains the handler function for retrieving all admin users.

use crate::{domain::models::Guest, errors::BResult, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// Handler for retrieving all admin users.
///
/// This function fetches all users with admin privileges from the database.
/// It's restricted to users who themselves have admin privileges.
///
/// # Arguments
///
/// * `state` - The application state, containing the guest CRUD operations.
/// * `admin` - The authenticated user making the request, used for authorization.
///
/// # Returns
///
/// Returns a `BResult` containing:
/// - On success: A `200 OK` status with a JSON array of all admin users.
/// - On unauthorized access: A `403 Forbidden` status with an error message.
///
/// # Errors
///
/// This function will return an error if:
/// - The requesting user is not an admin (403 Forbidden).
/// - The database query fails (wrapped in BResult).
///
/// # Security Considerations
///
/// - This endpoint should only be accessible to authenticated users with admin privileges.
/// - The list of admins could be sensitive information, so ensure it's transmitted securely.
/// - Consider implementing rate limiting to prevent abuse.
///
/// # Example
///
/// ```json
/// GET /admin/all
/// Authorization: Bearer <admin-token>
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
///     "name": "Admin User 1",
///     "email": "admin1@example.com",
///     "is_admin": true
///   },
///   {
///     "id": 2,
///     "name": "Admin User 2",
///     "email": "admin2@example.com",
///     "is_admin": true
///   }
/// ]
/// ```
///
/// Unauthorized response:
/// ```json
/// HTTP/1.1 403 Forbidden
/// Content-Type: text/plain
///
/// Only admins can view all admins
/// ```
pub async fn get_all_admins(
    State(state): State<AppState>,
    admin: Guest,
) -> BResult<impl IntoResponse> {
    if !admin.is_admin {
        return Ok((StatusCode::FORBIDDEN, "Only admins can view all admins").into_response());
    }

    let admins = state.guest_repo.get_all_admins().await?;
    Ok((StatusCode::OK, Json(admins)).into_response())
}

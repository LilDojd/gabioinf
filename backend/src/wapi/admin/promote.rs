//! Admin promotion handler.
//!
//! This module contains the handler function for promoting a user to admin status.

use crate::{
    domain::models::Guest,
    errors::BResult,
    repos::{GuestCriteria, Repository},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

/// Handler for promoting a user to admin status.
///
/// This function allows an existing admin to promote another user to admin status.
/// It's a sensitive operation that should be carefully controlled and audited.
///
/// # Arguments
///
/// * `state` - The application state, containing the guest CRUD operations.
/// * `admin` - The authenticated user making the request, used for authorization.
/// * `guest_id` - The ID of the user to be promoted to admin status.
///
/// # Returns
///
/// Returns a `BResult` containing:
/// - On success: A `200 OK` status with a JSON representation of the newly promoted admin.
/// - On unauthorized access: A `403 Forbidden` status with an error message.
///
/// # Errors
///
/// This function will return an error if:
/// - The requesting user is not an admin (403 Forbidden).
/// - The database operation to promote the user fails (wrapped in BResult).
/// - The specified user ID does not exist (should be handled by the CRUD operation).
///
/// # Security Considerations
///
/// - This endpoint should only be accessible to authenticated users with existing admin privileges.
/// - Implement strict audit logging for all promotion attempts, successful or not.
/// - Consider implementing a multi-step verification process for admin promotion.
/// - Ensure that the number of admins doesn't grow unnecessarily large.
///
/// # Example
///
/// ```json
/// POST /admin/promote/123
/// Authorization: Bearer <admin-token>
/// ```
///
/// Successful response:
/// ```json
/// HTTP/1.1 200 OK
/// Content-Type: application/json
///
/// {
///   "id": 123,
///   "name": "New Admin User",
///   "email": "newadmin@example.com",
///   "is_admin": true
/// }
/// ```
///
/// Unauthorized response:
/// ```json
/// HTTP/1.1 403 Forbidden
/// Content-Type: text/plain
///
/// Only admins can promote users
/// ```
pub async fn promote_to_admin(
    State(state): State<AppState>,
    admin: Guest,
    Path(guest_id): Path<i64>,
) -> BResult<impl IntoResponse> {
    if !admin.is_admin {
        tracing::warn!(
            "Non-admin user {:?} attempted to promote user {:?}",
            admin.id,
            guest_id
        );
        return Ok((StatusCode::FORBIDDEN, "Only admins can promote users").into_response());
    }

    if admin.id == guest_id.into() {
        tracing::warn!("Admin {:?} attempted to promote themselves", admin.id);
        return Ok((StatusCode::BAD_REQUEST, "Admins cannot promote themselves").into_response());
    }

    let mut guest = state
        .guest_repo
        .read(&GuestCriteria::WithGuestId(guest_id.into()))
        .await?;

    match guest.is_admin {
        true => {
            tracing::warn!(
                "Admin {:?} attempted to promote another admin {:?}",
                admin.id,
                guest.id
            );
            Ok((
                StatusCode::BAD_REQUEST,
                "User is already an admin and cannot be promoted",
            )
                .into_response())
        }
        false => {
            guest.is_admin = true;
            let promoted_guest = state.guest_repo.update(&guest).await?;

            tracing::info!(
                "User {:?} promoted to admin by admin {:?}",
                guest_id,
                admin.id
            );

            Ok((StatusCode::OK, Json(promoted_guest)).into_response())
        }
    }
}

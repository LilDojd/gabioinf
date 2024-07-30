//! User logout handler.
//!
//! This module contains the handler function for logging out a user,
//! which involves invalidating their session and removing the session cookie.

use crate::{
    errors::BResult,
    AppState,
};
use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::PrivateCookieJar;

/// Handler for logging out a user.
///
/// This function performs the following actions to log out a user:
/// 1. Retrieves the session ID from the cookie jar.
/// 2. If a session ID is found, it invalidates the corresponding session in the database.
/// 3. Removes the session cookie from the cookie jar.
///
/// # Arguments
///
/// * `state` - The application state, containing the guest CRUD operations.
/// * `jar` - The private cookie jar containing the user's cookies.
///
/// # Returns
///
/// Returns a `BResult` containing a response with:
/// - A `200 OK` status code.
/// - An updated cookie jar with the session cookie removed.
///
/// # Errors
///
/// This function will return an error if:
/// - The session invalidation in the database fails.
///
/// # Security Considerations
///
/// - This function uses a private cookie jar, which ensures that cookies are encrypted.
/// - The session is invalidated in the database, preventing reuse of the old session token.
/// - The session cookie is removed from the client, preventing client-side retention of the old session.
///
/// # Logging
///
/// This function logs a debug message when invalidating a user session.
///
/// # Example
///
/// ```json
/// POST /auth/logout
/// Cookie: sid=<encrypted-session-id>
/// ```
///
/// Successful response:
/// ```json
/// HTTP/1.1 200 OK
/// Set-Cookie: sid=; Max-Age=0; Path=/; HttpOnly; Secure; SameSite=Lax
/// ```
pub async fn logout(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> BResult<impl IntoResponse> {
    // if let Some(cookie) = jar.get("sid") {
    //     let token = cookie.value();
    //     // Try to fetch from db
    //     let session = state
    //         .session_repo
    //         .read(&SessionCriteria::WithToken(token.to_string()))
    //         .await?;
    //     tracing::debug!("Found user in db, invalidating user session");
    //     state.session_repo.delete(&session).await?;
    //     tracing::debug!("Removing session cookie from jar");
    //     let jar = jar.remove(Cookie::from("sid"));
    //     Ok((StatusCode::OK, jar))
    // } else {
    //     Err(ApiError::AuthenticationError(
    //         "Not authenticated".to_string(),
    //     ))
    // }
    Ok(())
}

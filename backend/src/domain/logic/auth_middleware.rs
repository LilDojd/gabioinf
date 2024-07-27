use crate::{
    errors::{ApiError, BResult},
    repos::{GuestCriteria, Repository, SessionCriteria},
    AppState,
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::PrivateCookieJar;

/// Middleware to authenticate users based on a session cookie.
///
/// This middleware performs the following steps:
/// 1. Extracts the session token from cookies.
/// 2. Retrieves session information from the database.
/// 3. Checks if the session has expired.
/// 4. Retrieves the guest information and adds it to the request extensions.
///
/// # Arguments
/// * `state` - The application state, containing database access
/// * `jar` - The private cookie jar for extracting the session cookie
/// * `req` - The incoming request
/// * `next` - The next middleware or handler in the chain
///
/// # Returns
/// * `Ok(Response)` if authentication is successful
/// * `Err(ApiError::AuthorizationError)` if no session cookie is present
/// * `Err(ApiError::AuthenticationError)` if the session has expired
/// * Other `Err(ApiError)` variants for database errors or other issues
pub async fn auth_middleware(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    mut req: Request,
    next: Next,
) -> BResult<Response> {
    // Extract token from cookies
    let token = jar
        .get("sid")
        .ok_or(ApiError::AuthorizationError(
            "No authorization cookie".to_string(),
        ))?
        .value()
        .to_string();

    tracing::debug!("Getting session info");
    let session = state
        .session_repo
        .read(&SessionCriteria::WithToken(token.clone()))
        .await
        .map_err(|_err| ApiError::AuthenticationError("You are not logged in".to_string()))?;

    // Check if the session has expired
    if chrono::Utc::now() > session.expires_at {
        state.session_repo.delete(&session).await?;
        return Err(ApiError::AuthenticationError(
            "Authentication token expired".to_string(),
        ));
    }

    tracing::debug!(
        "Session info: user_id: {:?}, expires_at: {:?}",
        session.user_id,
        session.expires_at
    );

    // Retrieve guest information and add it to request extensions
    let guest = state
        .guest_repo
        .read(&GuestCriteria::WithGuestId(session.user_id))
        .await?;
    req.extensions_mut().insert(guest);

    Ok(next.run(req).await)
}

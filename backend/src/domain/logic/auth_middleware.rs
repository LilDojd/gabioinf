use crate::{
    errors::{ApiError, BResult},
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
    let (user_id, expires_at) = state.guest_repo.get_session(&token).await?;

    // Check if the session has expired
    if chrono::Utc::now() > expires_at {
        state.guest_repo.invalidate_session(&token).await?;
        return Err(ApiError::AuthenticationError(
            "Authentication token expired".to_string(),
        ));
    }

    tracing::debug!(
        "Session info: user_id: {}, expires_at: {}",
        user_id,
        expires_at
    );

    // Retrieve guest information and add it to request extensions
    let guest = state.guest_repo.get_by_id(user_id).await?;
    req.extensions_mut().insert(guest);

    Ok(next.run(req).await)
}

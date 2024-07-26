use crate::{domain::models::Guest, errors::ApiError, AppState};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

/// Middleware to ensure that only admin users can access certain routes.
///
/// This middleware checks if the current user is authenticated and has admin privileges.
/// If the user is an admin, the request is allowed to proceed. Otherwise, an error is returned.
///
/// # Arguments
/// * `_state` - The application state (unused in this middleware)
/// * `req` - The incoming request
/// * `next` - The next middleware or handler in the chain
///
/// # Returns
/// * `Ok(Response)` if the user is an admin
/// * `Err(ApiError::AuthorizationError)` if the user is authenticated but not an admin
/// * `Err(ApiError::AuthenticationError)` if the user is not authenticated
pub async fn admin_middleware(
    State(_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Retrieve the Guest object from the request extensions
    let guest = req.extensions().get::<Guest>().cloned();

    match guest {
        Some(guest) if guest.is_admin => {
            // User is an admin, proceed with the request
            Ok(next.run(req).await)
        }
        Some(_) => {
            // User is authenticated but not an admin
            Err(ApiError::AuthorizationError(
                "Admin access required".to_string(),
            ))
        }
        None => Err(ApiError::AuthenticationError(
            "Login with github first".to_string(),
        )),
    }
}

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{domain::models::Guest, errors::ApiError, AppState};

pub async fn admin_middleware(
    State(_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
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

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{domain::models::Guest, errors::BackendError, AppState};

pub async fn admin_middleware(
    State(_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, BackendError> {
    let guest = req.extensions().get::<Guest>().cloned();

    match guest {
        Some(guest) if guest.is_admin => {
            // User is an admin, proceed with the request
            Ok(next.run(req).await)
        }
        Some(_) => {
            // User is authenticated but not an admin
            Err(BackendError::Forbidden("Admin access required".to_string()))
        }
        None => {
            // User is not authenticated
            Err(BackendError::Unauthorized)
        }
    }
}

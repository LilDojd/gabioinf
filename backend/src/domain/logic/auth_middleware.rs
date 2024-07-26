use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::PrivateCookieJar;

use crate::{
    errors::{ApiError, BResult},
    AppState,
};

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
    let (user_id, expires_at) = state.guest_crud.get_session(&token).await?;

    if chrono::Utc::now() > expires_at {
        state.guest_crud.invalidate_session(&token).await?;
        return Err(ApiError::AuthenticationError(
            "Authentication token expired".to_string(),
        ));
    }
    tracing::debug!(
        "Session info: user_id: {}, expires_at: {}",
        user_id,
        expires_at
    );
    let guest = state.guest_crud.get_by_id(user_id).await?;
    req.extensions_mut().insert(guest);

    Ok(next.run(req).await)
}

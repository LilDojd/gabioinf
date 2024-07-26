use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};

use crate::{errors::BResult, AppState};

pub async fn logout(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> BResult<impl IntoResponse> {
    if let Some(cookie) = jar.get("sid") {
        let token = cookie.value();
        tracing::debug!("Invalidating user session");
        state.guest_crud.invalidate_session(token).await?;
    }

    let jar = jar.remove(Cookie::from("sid"));
    Ok((StatusCode::OK, jar))
}

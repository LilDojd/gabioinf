//! API router configuration.
//!
//! This module defines the routing structure for the entire API, including
//! public routes, authenticated routes, and admin-only routes.
use crate::{
    config::AppConfig, domain::logic::{protected, router, AuthBackend, AuthSession},
    domain::models::PermissionTargets, AppState,
};
use askama::Template;
use askama_axum::IntoResponse;
use axum::{http, routing::get, Json, Router};
use axum_extra::extract::CookieJar;
use axum_login::{login_required, permission_required};
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::HeaderValue;
use http::Method;
use reqwest::StatusCode;
use serde_json::json;
use tower_http::cors::CorsLayer;
#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate;
pub async fn admin(auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.user {
        Some(_) => AdminTemplate {}.into_response(),
        None => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
/// Configures and returns the main API router.
///
/// This function sets up all routes for the application, including public routes,
/// authenticated routes, and admin-only routes. It also configures CORS and
/// attaches middleware where necessary.
///
/// # Arguments
///
/// * `state` - The shared application state.
/// * `oauth_client` - The OAuth client for authentication.
///
/// # Returns
///
/// Returns a configured `Router` instance ready to be served.
pub fn api_router(state: AppState, config: AppConfig) -> Router {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT])
        .allow_origin(state.domain.parse::<HeaderValue>().unwrap());
    let auth_router = crate::domain::logic::auth::router();
    let oauth_router = router();
    let admin_router = Router::new()
        .route("/admin", get(admin))
        .route_layer(
            permission_required!(
                AuthBackend, login_url = "/", PermissionTargets::MarkAsNaughty
            ),
        );
    Router::new()
        .route("/auth/status", get(auth_status))
        .merge(admin_router)
        .merge(auth_router)
        .merge(oauth_router)
        .layer(CorsLayer::very_permissive())
}
async fn auth_status(auth_session: AuthSession) -> impl IntoResponse {
    if auth_session.user.is_some() { StatusCode::OK } else { StatusCode::UNAUTHORIZED }
}

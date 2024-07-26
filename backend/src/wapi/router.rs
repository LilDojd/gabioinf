//! API router configuration.
//!
//! This module defines the routing structure for the entire API, including
//! public routes, authenticated routes, and admin-only routes.
use crate::{
    domain::logic::{admin_middleware, auth_middleware, github_auth, github_callback, protected},
    AppState,
};
use axum::{
    http,
    routing::{delete, get, post, put},
    Extension, Router,
};
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::HeaderValue;
use http::Method;
use oauth2::basic::BasicClient;
use tower_http::cors::CorsLayer;

use super::{
    admin_dashboard, create_entry, delete_entry, execute_sql_query, flag_as_naughty,
    get_all_admins, get_all_entries, get_guests, get_naughty_entries, logout, promote_to_admin,
    update_entry,
};

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
pub fn api_router(state: AppState, oauth_client: BasicClient) -> Router {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT])
        .allow_origin(state.domain.parse::<HeaderValue>().unwrap());

    let public_guestbook_router = Router::new()
        .route("/", get(get_all_entries))
        .route("/naughty", get(get_naughty_entries));

    let guestbook_router = Router::new()
        .route("/", post(create_entry))
        .route("/:id", delete(delete_entry))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let admin_guestbook_router = Router::new()
        .route("/:id/flag", post(flag_as_naughty))
        .route("/:id", put(update_entry))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            admin_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let admin_router = Router::new()
        .route("/admin", get(admin_dashboard))
        .route("/admin/promote/:guest_id", post(promote_to_admin))
        .route("/admin/all", get(get_all_admins))
        .route("/admin/query", post(execute_sql_query))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            admin_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let auth_router = Router::new()
        .route("/auth/github", get(github_auth))
        .route("/auth/authorized", get(github_callback))
        .route("/auth/logout", post(logout));

    let protected_router = Router::new()
        .route("/protected", get(protected))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let guests_router = Router::new().route("/all", get(get_guests));

    Router::new()
        .nest("/guestbook", guestbook_router)
        .nest("/guestbook", public_guestbook_router)
        .nest("/admin/guestbook", admin_guestbook_router)
        .nest("/", auth_router)
        .nest("/guests", guests_router)
        .nest("/", protected_router)
        .nest("/", admin_router)
        .with_state(state)
        .layer(cors)
        .layer(Extension(oauth_client))
}

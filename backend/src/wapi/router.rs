//! API router configuration.
//!
//! This module defines the routing structure for the entire API, including
//! public routes, authenticated routes, and admin-only routes.

use crate::{
    config::AppConfig,
    domain::logic::{
        auth_middleware, build_oauth_client, github_auth, github_callback, protected, RequiresAdmin,
    },
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
use tower_http::cors::CorsLayer;

use super::{
    admin_dashboard, create_entry, delete_entry, execute_sql_query, flag_as_naughty,
    get_all_admins, get_all_entries, get_naughty_entries, logout, promote_to_admin, update_entry,
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
pub fn api_router(state: AppState, config: AppConfig) -> Router {
    let oauth_client = build_oauth_client(
        state.clone().client_id,
        state.clone().client_secret,
        config.oauth.oauth_redirect_uri,
    );

    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT])
        .allow_origin(state.domain.parse::<HeaderValue>().unwrap());

    let public_router = Router::new()
        .route("/guestbook", get(get_all_entries))
        .route("/guestbook/naughty", get(get_naughty_entries));

    let auth_router = Router::new()
        // .route("/auth/github", get(github_auth))
        // .route("/auth/github/callback", get(github_callback))
        .route("/auth/logout", post(logout));

    let authenticated_router = Router::new()
        .route("/guestbook", post(create_entry))
        .route("/guestbook/:id", delete(delete_entry))
        .route("/protected", get(protected))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let admin_router = Router::new()
        .route("/guestbook/:id", put(update_entry))
        .route("/guestbook/:id/flag", post(flag_as_naughty))
        .route("/admin", get(admin_dashboard))
        .route("/admins", get(get_all_admins))
        .route("/admins/:id", post(promote_to_admin))
        .route("/queries", post(execute_sql_query))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(Extension(RequiresAdmin));

    Router::new()
        .merge(public_router)
        .merge(auth_router)
        .merge(authenticated_router)
        .merge(admin_router)
        .with_state(state)
        .layer(cors)
        .layer(Extension(oauth_client))
}

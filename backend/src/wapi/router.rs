use crate::{
    domain::logic::{github_auth, github_callback, protected},
    AppState,
};
use axum::{
    http,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::HeaderValue;
use http::Method;
use oauth2::basic::BasicClient;
use tower_http::cors::CorsLayer;

use super::get_guests;

pub fn api_router(state: AppState, oauth_client: BasicClient) -> Router {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT])
        .allow_origin(state.domain.parse::<HeaderValue>().unwrap());

    let guestbook_router = Router::new();
    // .route("/", get(guestbook))
    // .route("/sign", post(sign_guestbook))
    // .route("/:id/hide", patch(hide_entry))
    // .route("/:id/delete", delete(delete_entry));

    let auth_router = Router::new()
        .route("/auth/github", get(github_auth))
        .route("/auth/authorized", get(github_callback));

    let protected_router = Router::new().route("/protected", get(protected));

    let guests_router = Router::new().route("/all", get(get_guests));

    Router::new()
        .nest("/guestbook", guestbook_router)
        .nest("/", auth_router)
        .nest("/guests", guests_router)
        .nest("/", protected_router)
        .with_state(state)
        .layer(cors)
        .layer(Extension(oauth_client))
}

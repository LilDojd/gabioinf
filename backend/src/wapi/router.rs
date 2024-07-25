use crate::AppState;
use axum::{
    http,
    routing::{delete, get, patch, post},
    Router,
};
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::HeaderValue;
use http::Method;
use tower_http::cors::CorsLayer;

pub fn api_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT])
        .allow_origin(state.domain.parse::<HeaderValue>().unwrap());

    let guestbook_router = Router::new()
        .route("/", get(guestbook))
        .route("/sign", post(sign_guestbook))
        .route("/:id/hide", patch(hide_entry))
        .route("/:id/delete", delete(delete_entry));

    Router::new()
        .nest("/guestbook", guestbook_router)
        .with_state(state)
        .layer(cors)
}

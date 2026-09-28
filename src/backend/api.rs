//! `/v1/*`: the non-Dioxus HTTP surface (sign-in and OAuth callback) with
//! a timeout and per-visitor rate limiting.
use crate::backend::{auth, rate_limit::FlyClientIpExtractor};
use axum::{Router, http::StatusCode};
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfig;
use tower_http::timeout::TimeoutLayer;
/// Adds the sign-in routes with API-specific middleware.
pub fn api_router(
    governor_conf: Arc<GovernorConfig<FlyClientIpExtractor, NoOpMiddleware<QuantaInstant>>>,
) -> Router {
    auth::router().layer(
        ServiceBuilder::new()
            .layer(GovernorLayer::new(governor_conf))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::GATEWAY_TIMEOUT,
                Duration::from_secs(10),
            ))
            .into_inner(),
    )
}

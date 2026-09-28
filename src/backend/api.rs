//! `/v1/*`: the non-Dioxus HTTP surface (sign-in and OAuth callback) with
//! security headers, a timeout and per-visitor rate limiting.
use crate::backend::{auth, rate_limit::FlyClientIpExtractor};
use axum::{Router, http::StatusCode};
use axum_helmet::{
    CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet, HelmetLayer, OriginAgentCluster,
    ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions, XDNSPrefetchControl,
    XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies, XXSSProtection,
};
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
    let helmet_layer: HelmetLayer = generate_general_helmet_headers()
        .into_layer()
        .expect("static security headers must be valid");
    auth::router().layer(
        ServiceBuilder::new()
            .layer(GovernorLayer::new(governor_conf))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::GATEWAY_TIMEOUT,
                Duration::from_secs(10),
            ))
            .layer(helmet_layer)
            .into_inner(),
    )
}
fn generate_general_helmet_headers() -> Helmet {
    Helmet::new()
        .add(CrossOriginOpenerPolicy::same_origin())
        .add(CrossOriginResourcePolicy::same_origin())
        .add(OriginAgentCluster::new(true))
        .add(ReferrerPolicy::no_referrer())
        .add(
            StrictTransportSecurity::new()
                .max_age(15_552_000)
                .include_sub_domains(),
        )
        .add(XContentTypeOptions::nosniff())
        .add(XDNSPrefetchControl::off())
        .add(XDownloadOptions::noopen())
        .add(XFrameOptions::Deny)
        .add(XPermittedCrossDomainPolicies::none())
        .add(XXSSProtection::off())
}

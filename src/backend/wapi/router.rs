//! API router configuration.
//!
//! This module defines the routing structure for the entire API for the authenticated routes.
//! The rest is delegated to dioxus server functions
use crate::backend::db::ping_db;
use crate::backend::domain::logic;
use crate::backend::extractors::CookieExtractor;
use crate::backend::AppState;
use axum::body::Body;
use axum::error_handling::HandleErrorLayer;
use axum::http::{Response, StatusCode};
use axum::{http, Router};
use axum_helmet::{
    ContentSecurityPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet,
    HelmetLayer, OriginAgentCluster, ReferrerPolicy, StrictTransportSecurity,
    XContentTypeOptions, XDNSPrefetchControl, XDownloadOptions, XFrameOptions,
    XPermittedCrossDomainPolicies, XXSSProtection,
};
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::HeaderValue;
use http::Method;
use std::sync::Arc;
use tower::timeout::error::Elapsed;
use tower::{BoxError, ServiceBuilder};
use tower_governor::governor::GovernorConfig;
use tower_governor::GovernorLayer;
use tower_http::cors::CorsLayer;
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
pub fn api_router(
    state: AppState,
    governor_conf: Arc<GovernorConfig<CookieExtractor, NoOpMiddleware<QuantaInstant>>>,
) -> Router {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT])
        .allow_origin(state.domain.parse::<HeaderValue>().unwrap());
    let helmet_layer = HelmetLayer::new(generate_general_helmet_headers());
    let auth_router = logic::auth::router();
    let oauth_router = logic::oauth::router();
    let api_router = Router::new()
        .route("/ping", axum::routing::get(ping_db))
        .with_state(state)
        .merge(auth_router)
        .merge(oauth_router)
        .layer(cors);
    Router::new()
        .nest("/", api_router)
        .layer(
            ServiceBuilder::new()
                .layer(GovernorLayer {
                    config: governor_conf,
                })
                .layer(
                    HandleErrorLayer::new(|error: BoxError| async move {
                        if error.is::<Elapsed>() {
                            return Ok(StatusCode::REQUEST_TIMEOUT);
                        }
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }),
                )
                .timeout(std::time::Duration::from_secs(10))
                .layer(helmet_layer)
                .map_response(|mut res: Response<Body>| {
                    if res.headers().get("content-security-policy").is_none() {
                        res.headers_mut()
                            .insert(
                                "content-security-policy",
                                generate_default_csp()
                                    .to_string()
                                    .parse()
                                    .unwrap_or_else(|_| {
                                        dioxus_logger::tracing::error!(
                                            "Failed to parse default CSP"
                                        );
                                        HeaderValue::from_static(fallback_static_str_csp())
                                    }),
                            );
                    }
                    res
                })
                .into_inner(),
        )
}
/// Returns a default configuration of http security headers.
fn generate_general_helmet_headers() -> Helmet {
    Helmet::new()
        .add(CrossOriginOpenerPolicy::same_origin())
        .add(CrossOriginResourcePolicy::same_origin())
        .add(OriginAgentCluster::new(true))
        .add(ReferrerPolicy::no_referrer())
        .add(StrictTransportSecurity::new().max_age(15_552_000).include_sub_domains())
        .add(XContentTypeOptions::nosniff())
        .add(XDNSPrefetchControl::off())
        .add(XDownloadOptions::noopen())
        .add(XFrameOptions::Deny)
        .add(XPermittedCrossDomainPolicies::none())
        .add(XXSSProtection::off())
}
/// Returns a default strict Content Security Policy.
/// It's used whenever a custom CSP is not set.
fn generate_default_csp() -> ContentSecurityPolicy<'static> {
    ContentSecurityPolicy::new()
        .default_src(vec!["'self'"])
        .base_uri(vec!["'none'"])
        .font_src(vec!["'none'"])
        .form_action(vec!["'none'"])
        .frame_src(vec!["'none'"])
        .frame_ancestors(vec!["'none'"])
        .img_src(vec!["'none'"])
        .object_src(vec!["'none'"])
        .script_src(vec!["'none'"])
        .style_src(vec!["'none'"])
        .worker_src(vec!["'none'"])
        .upgrade_insecure_requests()
}
/// Returns a default strict Content Security Policy as a static string.
const fn fallback_static_str_csp() -> &'static str {
    "
    default-src 'self';
    base-uri 'none';
    font-src 'none';
    form-action 'none';
    frame-src 'none';
    frame-ancestors 'none';
    img-src 'none';
    object-src 'none';
    script-src 'none';
    style-src 'none';
    worker-src 'none';
    upgrade-insecure-requests;
    "
}

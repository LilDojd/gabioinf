//! `/v1/*`: the non-Dioxus HTTP surface (sign-in, OAuth callback, DB ping) with
//! CORS, security headers, a timeout and per-visitor rate limiting.
use crate::backend::AppState;
use crate::backend::db::ping_db;
use crate::backend::{auth, rate_limit::CookieExtractor};
use axum::body::Body;
use axum::error_handling::HandleErrorLayer;
use axum::http::{Response, StatusCode};
use axum::{Router, http};
use axum_helmet::{
    ContentSecurityPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet, HelmetLayer,
    OriginAgentCluster, ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions,
    XDNSPrefetchControl, XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies,
    XXSSProtection,
};
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use http::HeaderValue;
use http::Method;
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use std::sync::Arc;
use tower::timeout::error::Elapsed;
use tower::{BoxError, ServiceBuilder};
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfig;
use tower_http::cors::CorsLayer;
/// Adds the sign-in and database ping routes with API-specific middleware.
pub fn api_router(
    state: AppState,
    governor_conf: Arc<GovernorConfig<CookieExtractor, NoOpMiddleware<QuantaInstant>>>,
) -> Router {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT])
        .allow_origin(
            state
                .origin
                .parse::<HeaderValue>()
                .expect("the configured origin is a valid header value"),
        );
    let helmet_layer: HelmetLayer = generate_general_helmet_headers()
        .into_layer()
        .expect("static security headers must be valid");
    let api_router = Router::new()
        .route("/ping", axum::routing::get(ping_db))
        .with_state(state)
        .merge(auth::router())
        .layer(cors);
    Router::new().merge(api_router).layer(
        ServiceBuilder::new()
            .layer(GovernorLayer::new(governor_conf))
            .layer(HandleErrorLayer::new(handle_service_error))
            .timeout(std::time::Duration::from_secs(10))
            .layer(helmet_layer)
            .map_response(|mut res: Response<Body>| {
                if res.headers().get("content-security-policy").is_none() {
                    res.headers_mut().insert(
                        "content-security-policy",
                        generate_default_csp()
                            .to_string()
                            .parse()
                            .expect("the static CSP is a valid header value"),
                    );
                }
                res
            })
            .into_inner(),
    )
}
async fn handle_service_error(error: BoxError) -> StatusCode {
    if error.is::<Elapsed>() {
        StatusCode::REQUEST_TIMEOUT
    } else {
        tracing::error!(%error, "API middleware failed");
        StatusCode::INTERNAL_SERVER_ERROR
    }
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
fn generate_default_csp() -> ContentSecurityPolicy<'static> {
    ContentSecurityPolicy::new()
        .default_src(vec!["'self'"])
        .base_uri(vec!["'none'"])
        .font_src(vec!["'none'"])
        .form_action(vec!["'none'"])
        .frame_src(vec!["'none'"])
        .frame_ancestors(vec!["'none'"])
        .object_src(vec!["'none'"])
        .script_src(vec!["'self'", "'wasm-unsafe-eval'"])
        .style_src(vec!["'self'", "'unsafe-inline'"])
        .img_src(vec!["'self'", "data:", "blob:"])
        .connect_src(vec!["'self'", "https://api.github.com"])
        .worker_src(vec!["'none'"])
        .upgrade_insecure_requests()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, response::IntoResponse};

    #[tokio::test]
    async fn service_errors_return_only_public_statuses() {
        assert_eq!(
            handle_service_error(Box::new(Elapsed::new())).await,
            StatusCode::REQUEST_TIMEOUT
        );
        let response = handle_service_error("private middleware details".into())
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            to_bytes(response.into_body(), 1024)
                .await
                .unwrap()
                .is_empty()
        );
    }
}

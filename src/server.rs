use crate::backend::config::AppConfig;
use crate::backend::domain::logic::{build_oauth_client, AuthBackend};
use crate::backend::extractors::CookieExtractor;
use crate::backend::AppState;
use crate::App;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::{Router, ServiceExt};
use axum_helmet::{
    ContentSecurityPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet, HelmetLayer,
    OriginAgentCluster, ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions,
    XDNSPrefetchControl, XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies,
    XXSSProtection,
};
use axum_login::tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use dioxus::dioxus_core::VirtualDom;
use dioxus::fullstack::prelude::*;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::timeout::error::Elapsed;
use tower::{BoxError, Layer, ServiceBuilder};
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;
/// Returns a default configuration of http security headers.
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
/// Returns a default strict Content Security Policy.
/// It's used whenever a custom CSP is not set.
fn generate_default_csp() -> ContentSecurityPolicy<'static> {
    return ContentSecurityPolicy::new()
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
        .upgrade_insecure_requests();
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
pub async fn serve(
    cfg: impl Into<ServeConfig>,
    build_virtual_dom: impl Fn() -> VirtualDom + Send + Sync + 'static,
) {
    let postgres = sqlx::PgPool::connect("postgres://postgres:postgres@localhost:18577/gabioinf")
        .await
        .unwrap();
    tracing::info!("Running database migration..");
    sqlx::migrate!()
        .run(&postgres)
        .await
        .expect("Failed to run migrations");
    let config = AppConfig::new_local().expect("Failed to load local configuration");
    tracing::debug!("Loaded config: {:?}", config);
    let (domain, client_id, client_secret) = (
        "http://localhost:8000",
        "Iv23lin2YpB54ptGvRA3",
        "085c4392fe2e2bfdf9e670ed1420893120874e28",
    );
    let client = build_oauth_client(client_id, client_secret);
    let state = AppState::new(postgres.clone(), domain.to_string(), client.clone());
    let session_store = PostgresStore::new(postgres.clone());
    session_store.migrate().await.unwrap();
    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1)));
    let backend = AuthBackend::new(state.guest_repo.clone(), state.gp_repo.clone(), client);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.ratelimiting.requests_per_second)
            .burst_size(config.ratelimiting.burst_size)
            .key_extractor(CookieExtractor)
            .finish()
            .unwrap(),
    );
    let governor_limiter = governor_conf.limiter().clone();
    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            tracing::info!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    let helmet_layer = HelmetLayer::new(generate_general_helmet_headers());
    let app = Router::new()
        .serve_dioxus_application(cfg.into(), build_virtual_dom)
        .await
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(auth_layer),
        );
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("Listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

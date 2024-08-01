use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::response::Html;
use axum::Router;
use axum_helmet::{
    ContentSecurityPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy, Helmet, HelmetLayer,
    OriginAgentCluster, ReferrerPolicy, StrictTransportSecurity, XContentTypeOptions,
    XDNSPrefetchControl, XDownloadOptions, XFrameOptions, XPermittedCrossDomainPolicies,
    XXSSProtection,
};
use axum_login::tower_sessions::{session_store, ExpiredDeletion, Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use backend::config::AppConfig;
use backend::domain::logic::{build_oauth_client, AuthBackend};
use backend::extractors::CookieExtractor;
use backend::utils::grab_secrets;
use backend::{db::DbConnPool, wapi::api_router, AppState};
use shuttle_runtime::CustomError;
use std::net::SocketAddr;
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
#[derive(Debug)]
/// The custom service to be used in the shuttle runtime.
pub struct BackendService {
    /// The axum router.
    router: Router,
    /// The task to delete expired sessions.
    deletion_task: tokio::task::JoinHandle<Result<(), session_store::Error>>,
}
#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BackendService {
    async fn bind(mut self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        axum::serve(
            TcpListener::bind(addr).await.map_err(CustomError::new)?,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(CustomError::new)?;
        let _deletion = tokio::join!(self.deletion_task);
        Ok(())
    }
}
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] postgres: DbConnPool,
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<BackendService, shuttle_runtime::Error> {
    tracing::info!("Running database migration..");
    sqlx::migrate!()
        .run(&postgres)
        .await
        .expect("Failed to run migrations");
    let config = AppConfig::new_local().expect("Failed to load local configuration");
    tracing::debug!("Loaded config: {:?}", config);
    let (domain, client_id, client_secret) = grab_secrets(secrets);
    let client = build_oauth_client(client_id, client_secret);
    let state = AppState::new(postgres.clone(), domain, client.clone());
    let session_store = PostgresStore::new(postgres.clone());
    session_store.migrate().await.map_err(CustomError::new)?;
    let deletion_task = tokio::task::spawn(
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
    let api_router = api_router(state, config);
    let mut router = Router::new()
        .nest("/v1", api_router)
        .layer(
            ServiceBuilder::new()
                .layer(GovernorLayer {
                    config: governor_conf,
                })
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<Elapsed>() {
                        return Ok(StatusCode::REQUEST_TIMEOUT);
                    }
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {error}"),
                    ))
                }))
                .timeout(Duration::from_secs(10))
                .layer(auth_layer),
        )
        .nest_service(
            "/",
            ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
        );
    if cfg!(debug_assertions) {
        router = router.layer(tower_livereload::LiveReloadLayer::new());
    }
    Ok(BackendService {
        router,
        deletion_task,
    })
}
#[axum::debug_handler]
async fn homepage() -> Html<String> {
    Html(
        r#"
        <p>Welcome!</p>
        <a href="http://localhost:8000/v1/auth/github">
            Click here to sign into Github!
        </a>
    "#
        .to_string(),
    )
}
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

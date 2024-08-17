use crate::backend::config::AppConfig;
use crate::backend::domain::logic::oauth::build_oauth_client;
use crate::backend::domain::logic::AuthBackend;
use crate::backend::extractors::CookieExtractor;
use crate::backend::wapi::api_router;
use crate::backend::AppState;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::{Router, ServiceExt};
use axum_login::tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use dioxus::dioxus_core::{Element, VirtualDom};
use dioxus::fullstack::prelude::*;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower::timeout::error::Elapsed;
use tower::{BoxError, Layer, ServiceBuilder};
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;
pub async fn serve(cfg: impl Into<ServeConfig>, app: fn() -> Element) {
    let postgres = sqlx::PgPool::connect("postgres://postgres:postgres@localhost:18577/gabioinf")
        .await
        .unwrap();
    dioxus_logger::tracing::info!("Running database migration..");
    sqlx::migrate!()
        .run(&postgres)
        .await
        .expect("Failed to run migrations");
    let config = AppConfig::new_local().expect("Failed to load local configuration");
    dioxus_logger::tracing::debug!("Loaded config: {:?}", config);
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
            dioxus_logger::tracing::info!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    let app = Router::new()
        .serve_dioxus_application(cfg.into(), app)
        .nest("/v1/", api_router(state, config, governor_conf))
        .layer(auth_layer);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    dioxus_logger::tracing::info!("Listening on {}", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

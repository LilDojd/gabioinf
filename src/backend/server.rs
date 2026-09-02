use crate::backend::AppState;
use crate::backend::config::AppConfig;
use crate::backend::domain::logic::AuthBackend;
use crate::backend::domain::logic::oauth::build_oauth_client;
use crate::backend::extractors::CookieExtractor;
use crate::backend::wapi::api_router;
use axum::{Extension, Router};
use axum_login::AuthManagerLayerBuilder;
use axum_login::tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use dioxus::prelude::{DioxusRouterExt, Element, ServeConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::governor::GovernorConfigBuilder;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;
pub async fn serve(cfg: impl Into<ServeConfig>, dxapp: fn() -> Element) {
    let config = AppConfig::new_local().expect("Failed to load local configuration");
    dioxus_logger::tracing::info!("Loaded config: {:?}", config);
    let postgres = sqlx::PgPool::connect(config.database.url.as_str())
        .await
        .unwrap();
    dioxus_logger::tracing::info!("Running database migration..");
    sqlx::migrate!()
        .run(&postgres)
        .await
        .expect("Failed to run migrations");
    let (domain, client_id, client_secret) = (
        config.domain.as_str(),
        config.gabioinf.id.as_str(),
        config.gabioinf.secret.as_str(),
    );
    let client = build_oauth_client(client_id, client_secret, domain);
    let reqwest_client = reqwest::Client::new();
    let state = AppState::new(
        postgres.clone(),
        domain.to_string(),
        client.clone(),
        reqwest_client.clone(),
        config.session.secret.clone(),
    );
    let session_store = PostgresStore::new(postgres.clone());
    session_store.migrate().await.unwrap();
    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_signed(state.clone().key)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1)));
    let backend = AuthBackend::new(
        state.guest_repo.clone(),
        state.gp_repo.clone(),
        client,
        reqwest_client,
    );
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
        .serve_dioxus_application(cfg.into(), dxapp)
        .nest("/v1/", api_router(state.clone(), governor_conf))
        .layer(Extension(state))
        .layer(auth_layer);
    let address = dioxus::cli_config::fullstack_address_or_localhost();
    dioxus_logger::tracing::info!("Listening on {}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

use crate::backend::AppState;
use crate::backend::config::AppConfig;
use crate::backend::domain::logic::AuthBackend;
use crate::backend::domain::logic::oauth::build_oauth_client;
use crate::backend::extractors::CookieExtractor;
use crate::backend::wapi::api_router;
use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use axum_login::tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use dioxus::dioxus_core::Element;
use dioxus::server::{DioxusRouterExt, FullstackState, ServeConfig};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;
use tower_governor::governor::GovernorConfigBuilder;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;
pub async fn serve(cfg: ServeConfig, dxapp: fn() -> Element) {
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
    let api_state = state.clone();
    let serve_cfg = cfg.context_provider(move || state.clone());
    // Create the Dioxus fullstack state and router
    let fullstack_state = FullstackState::new(serve_cfg, dxapp);
    let app = Router::<FullstackState>::new()
        .register_server_functions()
        .serve_static_assets()
        .fallback(axum::routing::get(dioxus::server::render_handler))
        .with_state(fullstack_state)
        .nest("/v1/", api_router(api_state, governor_conf))
        .layer(auth_layer);
    use std::net::SocketAddr;
    let port = dioxus_cli_config::server_port().unwrap_or(8080);
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    dioxus_logger::tracing::info!("Listening on {}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

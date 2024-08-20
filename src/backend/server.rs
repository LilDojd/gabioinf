use crate::backend::config::AppConfig;
use crate::backend::domain::logic::oauth::build_oauth_client;
use crate::backend::domain::logic::AuthBackend;
use crate::backend::extractors::CookieExtractor;
use crate::backend::wapi::api_router;
use crate::backend::AppState;
use axum::Router;
use axum_login::tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use dioxus::dioxus_core::Element;
use dioxus::fullstack::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::governor::GovernorConfigBuilder;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;
pub async fn serve(cfg: impl Into<ServeConfig>, app: fn() -> Element) {
    let config = AppConfig::new_local().expect("Failed to load local configuration");
    dioxus_logger::tracing::info!("Loaded config: {:?}", config);
    let postgres = sqlx::PgPool::connect(config.database.url.as_str()).await.unwrap();
    dioxus_logger::tracing::info!("Running database migration..");
    sqlx::migrate!().run(&postgres).await.expect("Failed to run migrations");
    let (domain, client_id, client_secret) = (
        config.domain.as_str(),
        config.gabioinf.id.as_str(),
        config.gabioinf.secret.as_str(),
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
        .with_secure(true)
        .with_signed(state.clone().key)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1)));
    let backend = AuthBackend::new(
        state.guest_repo.clone(),
        state.gp_repo.clone(),
        client,
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
            dioxus_logger::tracing::info!(
                "rate limiting storage size: {}", governor_limiter.len()
            );
            governor_limiter.retain_recent();
        }
    });
    let cfg = cfg.into();
    let ssr_state = SSRState::new(&cfg);
    let app = Router::new()
        .nest("/v1/", api_router(state.clone(), governor_conf))
        .serve_static_assets()
        .register_server_functions_with_context(
            Arc::new(vec![Box::new(move || { Box::new(state.clone()) })]),
        )
        .fallback(
            axum::routing::get(render_handler)
                .with_state(
                    RenderHandleState::new(app)
                        .with_config(cfg)
                        .with_ssr_state(ssr_state),
                ),
        )
        .layer(auth_layer);
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    let listen_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    dioxus_logger::tracing::info!("Listening on {}", listen_address);
    axum_server::bind(listen_address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

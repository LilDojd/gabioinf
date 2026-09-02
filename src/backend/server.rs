use crate::backend::AppState;
use crate::backend::config::AppConfig;
use crate::backend::domain::logic::AuthBackend;
use crate::backend::domain::logic::oauth::build_oauth_client;
use crate::backend::extractors::CookieExtractor;
use crate::backend::health;
use crate::backend::observability;
use crate::backend::wapi::api_router;
use anyhow::Context;
use axum::{Extension, Router, extract::Request, middleware};
use axum_login::AuthManagerLayerBuilder;
use axum_login::tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use dioxus::prelude::{DioxusRouterExt, Element, ServeConfig};
use sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::governor::GovernorConfigBuilder;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;

pub async fn serve(cfg: impl Into<ServeConfig>, dxapp: fn() -> Element) -> anyhow::Result<()> {
    let config = AppConfig::new_local().context("failed to load local configuration")?;
    dioxus_logger::tracing::info!("Loaded config: {:?}", config);
    let postgres = sqlx::PgPool::connect(config.database.url.as_str())
        .await
        .context("failed to connect to database")?;
    dioxus_logger::tracing::info!("Running database migration..");
    sqlx::migrate!()
        .run(&postgres)
        .await
        .context("failed to run database migrations")?;
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
    session_store
        .migrate()
        .await
        .context("failed to migrate session store")?;
    let session_layer = SessionManagerLayer::new(session_store.clone())
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
            .context("invalid rate limiter configuration")?,
    );
    let governor_limiter = governor_conf.limiter().clone();
    let application = Router::new()
        .serve_dioxus_application(cfg.into(), dxapp)
        .nest("/v1/", api_router(state.clone(), governor_conf))
        .layer(middleware::from_fn(observability::sentry_user_context))
        .layer(Extension(state))
        .layer(auth_layer)
        .layer(SentryHttpLayer::new().enable_transaction())
        .layer(NewSentryLayer::<Request>::new_from_top());
    let app = Router::new()
        .nest("/health", health::router(postgres.clone()))
        .merge(application);
    let address = dioxus::cli_config::fullstack_address_or_localhost();
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind server to {address}"))?;

    let deletion_task = tokio::task::spawn(
        session_store.continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );
    let governor_task = tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            dioxus_logger::tracing::info!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    dioxus_logger::tracing::info!("Listening on {}", address);
    let result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await;

    deletion_task.abort();
    governor_task.abort();
    let _ = tokio::join!(deletion_task, governor_task);
    postgres.close().await;
    result.context("server failed")?;
    dioxus_logger::tracing::info!("Server stopped");
    Ok(())
}

async fn shutdown_signal() {
    shutdown_on(ctrl_c_signal(), terminate_signal()).await;
    dioxus_logger::tracing::info!("Shutdown signal received");
}

async fn shutdown_on(ctrl_c: impl Future<Output = ()>, terminate: impl Future<Output = ()>) {
    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}

async fn ctrl_c_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        dioxus_logger::tracing::error!(%error, "failed to listen for Ctrl-C");
    }
}

#[cfg(unix)]
async fn terminate_signal() {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Ok(mut signal) => {
            signal.recv().await;
        }
        Err(error) => {
            dioxus_logger::tracing::error!(%error, "failed to listen for SIGTERM");
        }
    }
}

#[cfg(not(unix))]
async fn terminate_signal() {
    std::future::pending::<()>().await;
}

#[cfg(test)]
mod tests {
    use super::shutdown_on;
    use std::future::{pending, ready};

    #[tokio::test]
    async fn shutdown_completes_for_ctrl_c() {
        shutdown_on(ready(()), pending()).await;
    }

    #[tokio::test]
    async fn shutdown_completes_for_sigterm() {
        shutdown_on(pending(), ready(())).await;
    }
}

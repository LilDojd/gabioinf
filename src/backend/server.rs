use crate::backend::config::AppConfig;
use crate::backend::domain::logic::oauth::build_oauth_client;
use crate::backend::domain::logic::AuthBackend;
use crate::backend::extractors::CookieExtractor;
use crate::backend::wapi::api_router;
use crate::backend::AppState;
use axum::extract::Request;
use axum::http::header;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::Router;
use axum_login::tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use dioxus::dioxus_core::Element;
use dioxus::fullstack::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_governor::governor::GovernorConfigBuilder;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;

use super::utils::{generate_etag, CachePolicy, CacheableResponse};
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
    let cfg = cfg.into();
    let ssr_state = SSRState::new(&cfg);

    let app = Router::new()
        .nest("/v1/", api_router(state.clone(), governor_conf))
        .serve_static_assets_cache()
        .register_server_functions_with_context(Arc::new(vec![Box::new(move || {
            Box::new(state.clone())
        })]))
        .fallback(
            axum::routing::get(render_handler)
                .with_state(RenderHandleState::new(cfg, dxapp).with_ssr_state(ssr_state)),
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

/// Get the path to the public assets directory to serve static files from
pub(crate) fn public_path() -> std::path::PathBuf {
    // The CLI always bundles static assets into the exe/public directory
    std::env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .unwrap()
        .join("public")
}

trait AxumAdapterExt<S> {
    fn serve_static_assets_cache(self) -> Self
    where
        Self: Sized;
}

impl<S> AxumAdapterExt<S> for Router<S>
where
    S: Send + Sync + Clone + 'static,
    Router<S>: DioxusRouterExt<S>,
{
    fn serve_static_assets_cache(mut self) -> Self {
        use tower_http::services::{ServeDir, ServeFile};

        let public_path = public_path();

        // Serve all files in public folder except index.html
        let dir = std::fs::read_dir(&public_path).unwrap_or_else(|e| {
            panic!(
                "Couldn't read public directory at {:?}: {}",
                &public_path, e
            )
        });

        for entry in dir.flatten() {
            let path = entry.path();
            if path.ends_with("index.html") {
                continue;
            }
            let route = path
                .strip_prefix(&public_path)
                .unwrap()
                .iter()
                .map(|segment| {
                    segment.to_str().unwrap_or_else(|| {
                        panic!("Failed to convert path segment {:?} to string", segment)
                    })
                })
                .collect::<Vec<_>>()
                .join("/");
            let route = format!("/{}", route);
            if path.is_dir() {
                self = self.nest_service(&route, ServeDir::new(path).precompressed_br());
            } else {
                self = self.nest_service(
                    &route,
                    ServiceBuilder::new()
                        .layer(middleware::from_fn(set_static_cache_control))
                        .service(ServeFile::new(path).precompressed_br()),
                );
            }
        }

        self
    }
}

async fn set_static_cache_control(request: Request, next: Next) -> Response {
    // Grab all we need from request before it is consumed
    let etag = generate_etag(request.uri().path());

    let mut response = next.run(request).await;

    // Set Cache-Control header
    response.set_cache_policy(CachePolicy::Revalidate {
        ttl: std::time::Duration::from_secs(604800),
        stale_ttl: Some(std::time::Duration::from_secs(86400)),
    });
    response.set_etag(etag);

    // Remove unnecessary headers
    response.headers_mut().remove(header::LAST_MODIFIED);
    response.headers_mut().remove(header::EXPIRES);
    response.headers_mut().remove(header::PRAGMA);

    response
}

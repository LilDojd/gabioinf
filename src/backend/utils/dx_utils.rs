use axum::{error_handling::HandleErrorLayer, http::StatusCode, BoxError, Router};
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_etag_cache::{const_lru_provider::ConstLruProvider, EtagCacheLayer};

/// Get the path to the public assets directory to serve static files from
pub(crate) fn public_path() -> PathBuf {
    // The CLI always bundles static assets into the exe/public directory
    std::env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .unwrap()
        .join("public")
}

pub(crate) trait LocalRouterExt<S> {
    fn serve_static_assets_etagged(self) -> Self
    where
        Self: Sized;
}

impl<S> LocalRouterExt<S> for Router<S>
where
    S: Send + Sync + Clone + 'static,
{
    fn serve_static_assets_etagged(mut self) -> Self
    where
        Self: Sized,
    {
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
                self = self.nest_service(&route, ServeFile::new(path).precompressed_br());
            }
        }

        let etag_cache_layer =
            EtagCacheLayer::with_default_predicate(ConstLruProvider::<_, _, 255, u8>::init(5));

        let etag_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_etag_cache_layer_err))
            .layer(etag_cache_layer);

        self.layer(etag_service)
    }
}

async fn handle_etag_cache_layer_err<T: Into<BoxError>>(err: T) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.into().to_string())
}

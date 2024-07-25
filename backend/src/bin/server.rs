use axum::Router;
use backend::{db::DbConnPool, wapi::api_router, AppState};
mod shuttle_utils;
use shuttle_utils::grab_secrets;
use tower_http::services::{ServeDir, ServeFile};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] postgres: DbConnPool,
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&postgres)
        .await
        .expect("Failed to run migrations");

    let (domain) = grab_secrets(secrets);

    let state = AppState::new(postgres, domain);

    let api_router = api_router(state);

    let mut router = Router::new().nest("/v1", api_router).nest_service(
        "/",
        ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
    );

    if cfg!(debug_assertions) {
        router = router.layer(tower_livereload::LiveReloadLayer::new());
    }

    Ok(router.into())
}


use axum::response::Html;
use axum::routing::get;
use axum::Router;
use backend::config::AppConfig;
use backend::utils::grab_secrets;
use backend::{db::DbConnPool, wapi::api_router, AppState};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] postgres: DbConnPool,
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    tracing::info!("Running database migration..");
    sqlx::migrate!()
        .run(&postgres)
        .await
        .expect("Failed to run migrations");

    let config = AppConfig::new_local().expect("Failed to load local configuration");
    tracing::debug!("Loaded config: {:?}", config);

    let (domain, client_id, client_secret) = grab_secrets(secrets);

    let state = AppState::new(postgres, domain, client_secret, client_id);

    let api_router = api_router(state, config);

    let homepage_router = Router::new().route("/", get(homepage));

    let mut router = Router::new()
        .nest("/v1", api_router)
        .nest("/", homepage_router);
    // .nest_service(
    //     "/",
    //     ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
    // );

    if cfg!(debug_assertions) {
        router = router.layer(tower_livereload::LiveReloadLayer::new());
    }

    Ok(router.into())
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

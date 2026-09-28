/// Build an authenticated request context for posting tests without GitHub or a running server.
pub(crate) async fn moderation_test_context(
    pool: sqlx::PgPool,
) -> dioxus::fullstack::FullstackContext {
    use crate::{
        backend::auth::{AuthBackend, AuthSession, build_oauth_client},
        shared::models::{GithubId, NewGuest},
    };
    use axum::{
        extract::FromRequestParts,
        http::{Request, Response},
    };
    use dioxus::fullstack::FullstackContext;
    use std::{convert::Infallible, sync::Arc};
    use tower::{ServiceExt, service_fn};
    use tower_sessions::{MemoryStore, Session};

    let state = crate::backend::AppState::new(pool);
    let guest = state
        .guest_repo
        .upsert(&NewGuest {
            id: GithubId(1),
            username: "moderation-test".into(),
            name: Some("Moderation Test".into()),
        })
        .await
        .unwrap();
    let backend = AuthBackend::new(
        state.guest_repo.clone(),
        build_oauth_client("test-client", "test-secret", "https://example.test"),
        reqwest::Client::new(),
    );
    // Let axum-login create its session extension, then retain the real request parts.
    let capture = service_fn(|request: Request<()>| async move {
        Ok::<_, Infallible>(Response::new(Some(request.into_parts().0)))
    });
    let mut request = Request::new(());
    request
        .extensions_mut()
        .insert(Session::new(None, Arc::new(MemoryStore::default()), None));
    let mut parts = axum_login::AuthManager::new(capture, backend, "moderation-test")
        .oneshot(request)
        .await
        .unwrap()
        .into_body()
        .unwrap();
    let session = AuthSession::from_request_parts(&mut parts, &())
        .await
        .unwrap();
    session.login(&guest).await.unwrap();
    parts.extensions.insert(session);
    parts.extensions.insert(state);
    FullstackContext::new(parts)
}

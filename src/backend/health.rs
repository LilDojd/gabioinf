use crate::backend::db::DbConnPool;
use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};

pub fn router(db: DbConnPool) -> Router {
    Router::new()
        .route("/live", get(liveness))
        .route("/ready", get(readiness))
        .with_state(db)
}

async fn liveness() -> impl IntoResponse {
    (StatusCode::OK, "alive")
}

async fn readiness(State(db): State<DbConnPool>) -> impl IntoResponse {
    if sqlx::query!("SELECT 1 AS one").fetch_one(&db).await.is_ok() {
        (StatusCode::OK, "ready")
    } else {
        tracing::warn!("readiness check failed");
        (StatusCode::SERVICE_UNAVAILABLE, "not ready")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::Request};
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use tower::ServiceExt;

    fn unavailable_pool() -> DbConnPool {
        PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(50))
            .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/postgres")
            .expect("test database URL must be valid")
    }

    async fn body(response: axum::response::Response) -> String {
        let bytes = to_bytes(response.into_body(), 32)
            .await
            .expect("health response body must be readable");
        String::from_utf8(bytes.to_vec()).expect("health response must be UTF-8")
    }

    #[tokio::test]
    async fn liveness_does_not_require_database() {
        let response = router(unavailable_pool())
            .oneshot(
                Request::get("/live")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body(response).await, "alive");
    }

    #[tokio::test]
    async fn readiness_hides_database_failure() {
        let response = router(unavailable_pool())
            .oneshot(
                Request::get("/ready")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body(response).await, "not ready");
    }
}

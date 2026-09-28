use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

pub fn router() -> Router {
    Router::new().route("/live", get(liveness))
}

async fn liveness() -> impl IntoResponse {
    (StatusCode::OK, "alive")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn liveness_does_not_require_database() {
        let response = router()
            .oneshot(
                Request::get("/live")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(to_bytes(response.into_body(), 32).await.unwrap(), "alive");
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub(crate) type BResult<T> = Result<T, ApiError>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("authentication failed: {0}")]
    Authentication(String),
    #[error("external service error: {0}")]
    ExternalService(#[from] reqwest::Error),
    #[error("invalid database data: {0}")]
    InvalidData(&'static str),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Authentication(_) => StatusCode::UNAUTHORIZED,
            Self::ExternalService(_) => StatusCode::BAD_GATEWAY,
            Self::Database(_) | Self::InvalidData(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        tracing::error!(error = %self, "API request failed");
        (
            status,
            status.canonical_reason().unwrap_or("Request failed"),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn responses_preserve_status_without_exposing_internal_details() {
        let cases = [
            (
                ApiError::Database(sqlx::Error::Protocol("private database details".into())),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                ApiError::InvalidData("private row details"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                ApiError::Authentication("private token details".into()),
                StatusCode::UNAUTHORIZED,
            ),
            (
                ApiError::ExternalService(
                    reqwest::Client::new().get("not a URL").build().unwrap_err(),
                ),
                StatusCode::BAD_GATEWAY,
            ),
        ];
        for (error, status) in cases {
            let response = error.into_response();
            assert_eq!(response.status(), status);
            let body = to_bytes(response.into_body(), 1024).await.unwrap();
            assert_eq!(body.as_ref(), status.canonical_reason().unwrap().as_bytes());
        }
    }
}

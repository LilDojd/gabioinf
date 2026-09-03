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
        (status, self.to_string()).into_response()
    }
}

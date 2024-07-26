//! Error types for the backend
//!
//! This module contains the error types for the backend in the [`ApiError`] enum.
//!
//! Additionaly, [`ApiError`] implements the [`IntoResponse`] trait, which allows it to be
//! straitforwardly converted into an axum response.
//!
//! [`IntoResponse`]: axum::response::IntoResponse

use axum::http::StatusCode;
use axum::response::IntoResponse;
use thiserror::Error;

pub(crate) type BResult<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("Resource already exists: {0}")]
    AlreadyExistsError(String),

    #[error("Invalid input: {0}")]
    ValidationError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("Encountered an error trying to convert an infallible value: {0}")]
    FromRequestPartsError(#[from] std::convert::Infallible),

    #[error("Unhandled error: {0}")]
    UnhandledError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let response = match self {
            Self::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::AuthenticationError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            Self::AuthorizationError(e) => (StatusCode::FORBIDDEN, e.to_string()),
            Self::NotFoundError(e) => (StatusCode::NOT_FOUND, e.to_string()),
            Self::AlreadyExistsError(e) => (StatusCode::CONFLICT, e.to_string()),
            Self::ValidationError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::ExternalServiceError(e) => (StatusCode::BAD_GATEWAY, e.to_string()),
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            Self::UnhandledError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::FromRequestPartsError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        response.into_response()
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::ExternalServiceError(e.to_string())
    }
}

type WeirdOauthError = oauth2::RequestTokenError<
    oauth2::reqwest::Error<reqwest::Error>,
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
>;

impl From<WeirdOauthError> for ApiError {
    fn from(e: WeirdOauthError) -> Self {
        Self::ExternalServiceError(e.to_string())
    }
}

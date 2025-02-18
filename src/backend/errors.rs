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
/// A Result type alias using ApiError as the error type
pub(crate) type BResult<T> = std::result::Result<T, ApiError>;
/// Represents all possible errors returned by this library
#[derive(Debug, Error)]
pub enum ApiError {
    /// Represents database-related errors
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    /// Represents authentication failures
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    /// Represents authorization failures
    #[error("Authorization failed: {0}")]
    AuthorizationError(String),
    /// Represents errors when a requested resource is not found
    #[error("Resource not found: {0}")]
    NotFoundError(String),
    /// Represents errors when attempting to create a resource that already exists
    #[error("Resource already exists: {0}")]
    AlreadyExistsError(String),
    /// Represents validation errors for input data
    #[error("Invalid input: {0}")]
    ValidationError(String),
    /// Represents errors from external services
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    /// Represents internal server errors
    #[error("Internal server error")]
    InternalServerError,
    /// Represents errors from infallible conversions (should never happen)
    #[error("Encountered an error trying to convert an infallible value: {0}")]
    FromRequestPartsError(#[from] std::convert::Infallible),
    #[error("Not implemented: {0}")]
    NotImplementedErrpr(String),
    /// Represents unhandled errors
    #[error("Unhandled error: {0}")]
    UnhandledError(String),
}
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let response = match self {
            Self::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DatabaseError: {}", e),
            ),
            Self::AuthenticationError(e) => (
                StatusCode::UNAUTHORIZED,
                format!("AuthenticationError: {}", e),
            ),
            Self::AuthorizationError(e) => {
                (StatusCode::FORBIDDEN, format!("AuthorizationError: {}", e))
            }
            Self::NotFoundError(e) => (StatusCode::NOT_FOUND, format!("NotFoundError: {}", e)),
            Self::AlreadyExistsError(e) => {
                (StatusCode::CONFLICT, format!("AlreadyExistsError: {}", e))
            }
            Self::ValidationError(e) => {
                (StatusCode::BAD_REQUEST, format!("ValidationError: {}", e))
            }
            Self::ExternalServiceError(e) => (
                StatusCode::BAD_GATEWAY,
                format!("ExternalServiceError: {}", e),
            ),
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalServerError: Internal server error".to_string(),
            ),
            Self::UnhandledError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("UnhandledError: {}", e),
            ),
            Self::NotImplementedErrpr(e) => (
                StatusCode::NOT_IMPLEMENTED,
                format!("NotImplementedError: {}", e),
            ),
            Self::FromRequestPartsError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("FromRequestPartsError: {}", e),
            ),
        };
        response.into_response()
    }
}
/// Implements conversion from [`reqwest::Error`] to [`ApiError`]
impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::ExternalServiceError(e.to_string())
    }
}
/// Type alias for a complex OAuth error type
type WeirdOauthError = oauth2::RequestTokenError<
    oauth2::reqwest::Error,
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
>;
/// Implements conversion from [`WeirdOauthError`] to [`ApiError`]
impl From<WeirdOauthError> for ApiError {
    fn from(e: WeirdOauthError) -> Self {
        Self::ExternalServiceError(e.to_string())
    }
}
impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::UnhandledError(e.to_string())
    }
}

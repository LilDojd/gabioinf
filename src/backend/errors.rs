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

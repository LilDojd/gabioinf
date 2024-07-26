use axum::http::StatusCode;
use axum::response::IntoResponse;
use thiserror::Error;

pub(crate) type BResult<T> = std::result::Result<T, BackendError>;

#[derive(Debug)]
pub enum UseCase {
    UserRegister,
    UserLogin,
    UpdateUser,
    Placeholder,
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("SQL error: {0}")]
    SQLErr(#[from] sqlx::Error),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal error")]
    InternalErr,

    #[error("{0} already exists")]
    AlreadyExists(String),

    #[error("{0} not found")]
    NotFound(String),

    #[error("Unhandled error")]
    UnhandledErr,

    #[error("OAuth token error: {0}")]
    TokenErr(String),

    #[error("Reqwest HTTP error: {0}")]
    ReqwestFetchErr(#[from] reqwest::Error),

    #[error("Called Option::unwrap on None value")]
    OptionErr,

    #[error("Encountered an error trying to convert an infallible value: {0}")]
    FromRequestPartsError(#[from] std::convert::Infallible),
}

impl From<(sqlx::Error, UseCase)> for BackendError {
    fn from(value: (sqlx::Error, UseCase)) -> Self {
        tracing::debug!("from((sqlx::Error, UseCase)): value={:?}", value);
        let (err, use_case) = value;
        match use_case {
            UseCase::UserRegister => {
                let db_err = err.into_database_error();
                match db_err {
                    Some(e) => {
                        e.code()
                            .map_or(BackendError::InternalErr, |code| match code.as_ref() {
                                "23505" => BackendError::AlreadyExists("User".to_string()),
                                _ => BackendError::InternalErr,
                            })
                    }
                    None => BackendError::InternalErr,
                }
            }
            UseCase::UserLogin => match &err {
                sqlx::Error::RowNotFound => BackendError::Unauthorized,
                _ => BackendError::InternalErr,
            },
            UseCase::UpdateUser => todo!(),
            _ => BackendError::UnhandledErr,
        }
    }
}

impl IntoResponse for BackendError {
    fn into_response(self) -> axum::response::Response {
        let response = match self {
            Self::SQLErr(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            Self::InternalErr => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            Self::AlreadyExists(e) => (StatusCode::CONFLICT, format!("{} already exists", e)),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, format!("{} not found", e)),
            Self::UnhandledErr => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unhandled error".to_string(),
            ),
            Self::TokenErr(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::ReqwestFetchErr(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::OptionErr => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Called Option::unwrap on None value".to_string(),
            ),
            Self::FromRequestPartsError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        response.into_response()
    }
}

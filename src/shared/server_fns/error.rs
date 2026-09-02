use dioxus::prelude::dioxus_fullstack::AsStatusCode;
use dioxus::prelude::{ServerFnError, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors that can safely cross the server-function boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum ServerError {
    Validation(String),
    InvalidRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    Conflict,
    Unavailable,
    Internal,
}

impl ServerError {
    #[cfg(feature = "server")]
    pub(crate) fn internal(context: &'static str, error: impl fmt::Debug) -> Self {
        dioxus_logger::tracing::error!(context, error = ?error, "Server function failed");
        Self::Internal
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Validation(message) => message,
            Self::InvalidRequest => "The request was invalid",
            Self::Unauthenticated => "Sign in before changing the guestbook",
            Self::Forbidden => "You are not allowed to perform this action",
            Self::NotFound => "Signature no longer exists",
            Self::Conflict => "You have already signed the guestbook",
            Self::Unavailable => "The service is temporarily unavailable",
            Self::Internal => "An internal error occurred",
        })
    }
}

impl std::error::Error for ServerError {}

impl From<ServerFnError> for ServerError {
    fn from(error: ServerFnError) -> Self {
        match error {
            ServerFnError::Args(_) | ServerFnError::MissingArg(_) => Self::InvalidRequest,
            ServerFnError::Request(_) => Self::Unavailable,
            _ => Self::Internal,
        }
    }
}

impl AsStatusCode for ServerError {
    fn as_status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) | Self::InvalidRequest => StatusCode::BAD_REQUEST,
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_map_to_http_statuses() {
        assert_eq!(
            ServerError::Validation("invalid message".into()).as_status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ServerError::InvalidRequest.as_status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ServerError::Unauthenticated.as_status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ServerError::Forbidden.as_status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ServerError::NotFound.as_status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(ServerError::Conflict.as_status_code(), StatusCode::CONFLICT);
        assert_eq!(
            ServerError::Unavailable.as_status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            ServerError::Internal.as_status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn errors_round_trip_through_json() {
        let error = ServerError::Validation("Message must not be empty".into());
        let json = serde_json::to_string(&error).unwrap();

        assert_eq!(serde_json::from_str::<ServerError>(&json).unwrap(), error);
    }

    #[test]
    fn framework_errors_do_not_expose_internal_details() {
        let error = ServerError::from(ServerFnError::ServerError {
            message: "database password leaked here".into(),
            code: 500,
            details: None,
        });
        let json = serde_json::to_string(&error).unwrap();

        assert_eq!(error, ServerError::Internal);
        assert!(!json.contains("password"));
    }
}

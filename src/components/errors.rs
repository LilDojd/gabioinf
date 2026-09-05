use crate::shared::server_fns::ServerError;

pub(crate) fn server_error_message(error: &ServerError, fallback: &str) -> String {
    match error {
        ServerError::Internal | ServerError::Unavailable => fallback.to_string(),
        error => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_validation_but_hides_internal_errors() {
        assert_eq!(
            server_error_message(
                &ServerError::Validation("Message is required".to_string()),
                "fallback"
            ),
            "Message is required"
        );
        assert_eq!(
            server_error_message(&ServerError::Internal, "Try again"),
            "Try again"
        );
    }
}

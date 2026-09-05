use rustrict::{CensorStr, Type};
use validator::ValidationError;

/// Checks all rustrict categories at SEVERE only; mild and moderate content is allowed.
pub fn contains_severe_content(text: &str) -> bool {
    text.is(Type::SEVERE)
}

pub fn validate_no_severe_content<S: AsRef<str>>(text: S) -> Result<(), ValidationError> {
    if contains_severe_content(text.as_ref()) {
        Err(ValidationError::new("offensive_content"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_clean_mild_and_moderate_content() {
        for text in [
            "This is a clean message",
            "This is a bad word: crap",
            "F u c k",
        ] {
            assert!(!contains_severe_content(text), "{text:?}");
            assert!(validate_no_severe_content(text).is_ok(), "{text:?}");
        }
    }

    #[test]
    fn rejects_severe_content() {
        let text = "i hope you die";
        assert!(contains_severe_content(text));
        assert_eq!(
            validate_no_severe_content(text).unwrap_err().code,
            "offensive_content"
        );
    }
}

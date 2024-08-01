use rustrict::{CensorStr, Type};
use validator::ValidationError;
/// Checks if the given text contains profanity.
pub fn is_severely_unappropriate(text: &str) -> bool {
    text.is(Type::SEVERE)
}
/// Censors any profanity in the given text.
pub fn censor_profanity(text: &str) -> String {
    text.censor()
}
pub fn validate_not_offensive<S: AsRef<str>>(text: S) -> Result<(), ValidationError> {
    if is_severely_unappropriate(text.as_ref()) {
        Err(ValidationError::new("offensive_content"))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_contains_profanity() {
        assert!(!is_severely_unappropriate("This is a bad word: crap"));
        assert!(!is_severely_unappropriate("F u c k"));
        assert!(!is_severely_unappropriate("This is a clean message"));
    }
    #[test]
    fn test_censor_profanity() {
        assert_eq!(censor_profanity("This is crap"), "This is c***");
        assert_eq!(censor_profanity("F u c k"), "F******");
        assert_eq!(censor_profanity("This is clean"), "This is clean");
    }
}

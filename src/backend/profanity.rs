use rustrict::{CensorStr, Type};

/// Checks all rustrict categories at SEVERE only; mild and moderate content is allowed.
pub fn contains_severe_content(text: &str) -> bool {
    text.is(Type::SEVERE)
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
        }
    }

    #[test]
    fn rejects_severe_content() {
        let text = "i hope you die";
        assert!(contains_severe_content(text));
    }
}

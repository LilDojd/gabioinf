use crate::{
    backend::{markdown, profanity::contains_severe_content},
    shared::{
        models::{COMMENT_MAX, GUESTBOOK_MESSAGE_MAX},
        server_fns::{CreateEntryRequest, ServerError},
    },
};
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub(crate) fn comment_body(body: String) -> Result<(String, String), ServerError> {
    let body = body.trim().to_string();
    if !(1..=COMMENT_MAX).contains(&body.chars().count()) {
        return Err(ServerError::Validation(format!(
            "Comment must be between 1 and {COMMENT_MAX} characters"
        )));
    }
    let offensive = || ServerError::Validation("Comment contains offensive content".to_string());
    if contains_severe_content(&body) {
        return Err(offensive());
    }
    let (body_html, visible_text) = markdown::render_with_text(&body)
        .map_err(|error| ServerError::Validation(format!("Invalid Markdown: {error}")))?;
    if contains_severe_content(&visible_text) {
        return Err(offensive());
    }
    Ok((body, body_html))
}

const MAX_SIGNATURE_BYTES: usize = 256 * 1024;

fn is_small_png(encoded: &str) -> bool {
    encoded.len() <= MAX_SIGNATURE_BYTES.div_ceil(3) * 4
        && STANDARD
            .decode(encoded)
            .is_ok_and(|png| png.starts_with(b"\x89PNG\r\n\x1a\n"))
}

pub(crate) fn guestbook_entry(
    mut payload: CreateEntryRequest,
) -> Result<CreateEntryRequest, ServerError> {
    payload.message = payload.message.trim().to_string();
    if !(1..=GUESTBOOK_MESSAGE_MAX).contains(&payload.message.chars().count()) {
        return Err(ServerError::Validation(format!(
            "Message must be between 1 and {GUESTBOOK_MESSAGE_MAX} characters"
        )));
    }
    if contains_severe_content(&payload.message) {
        return Err(ServerError::Validation(
            "Message contains offensive content".to_string(),
        ));
    }
    if payload
        .signature
        .as_deref()
        .is_some_and(|signature| !is_small_png(signature))
    {
        return Err(ServerError::Validation(
            "Signature must be a PNG drawing under 256 KB".to_string(),
        ));
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(message: &str) -> CreateEntryRequest {
        CreateEntryRequest {
            message: message.to_string(),
            signature: None,
        }
    }

    #[test]
    fn validation_trims_and_renders_safe_markdown() {
        let (body, html) = comment_body("  **hello**  ".to_string()).unwrap();

        assert_eq!(body, "**hello**");
        assert_eq!(html, "<p><strong>hello</strong></p>\n");
    }

    #[test]
    fn validation_allows_clean_mild_and_moderate_comments() {
        for text in [
            "This is a clean message",
            "This is a bad word: crap",
            "F u c k",
        ] {
            let (body, html) = comment_body(format!("  {text}  ")).unwrap();
            assert_eq!(body, text);
            assert_eq!(html, format!("<p>{text}</p>\n"));
        }
    }

    #[test]
    fn validation_rejects_severe_comments_with_user_facing_error() {
        assert_eq!(
            comment_body("  i hope you die  ".to_string()),
            Err(ServerError::Validation(
                "Comment contains offensive content".to_string()
            ))
        );
    }

    #[test]
    fn validation_rejects_equivalent_severe_text_in_entities_and_links() {
        for body in [
            "i h&#111;p&#101; y&#111;u d&#105;&#101;",
            "i h&#x6f;p&#x65; y&#x6f;u d&#x69;&#x65;",
            "i ho[pe](https://example.com) you die",
            "i ho[**pe**](https://example.com) you die",
        ] {
            assert_eq!(
                comment_body(body.to_string()),
                Err(ServerError::Validation(
                    "Comment contains offensive content".to_string()
                )),
                "{body:?}"
            );
        }
    }

    #[test]
    fn validation_still_rejects_severe_raw_text_outside_link_labels() {
        assert_eq!(
            comment_body("[hello](https://example.com \"i hope you die\")".to_string()),
            Err(ServerError::Validation(
                "Comment contains offensive content".to_string()
            ))
        );
    }

    #[test]
    fn validation_allows_formatted_clean_mild_and_moderate_comments() {
        for body in [
            "**hello** [world](https://example.com)",
            "This is [crap](https://example.com)",
            "This is [cr&#97;p](https://example.com)",
            "**F u c k**",
        ] {
            let (stored, _) =
                comment_body(body.to_string()).unwrap_or_else(|error| panic!("{body:?}: {error}"));
            assert_eq!(stored, body);
        }
    }

    #[test]
    fn validation_keeps_encoded_html_as_escaped_text() {
        let (_, html) = comment_body("&lt;script&gt;alert(1)&lt;/script&gt;".to_string()).unwrap();
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn validation_rejects_unsafe_markdown_links() {
        assert!(matches!(
            comment_body("[click](javascript:alert(1))".to_string()),
            Err(ServerError::Validation(message)) if message.starts_with("Invalid Markdown:")
        ));
    }

    #[test]
    fn validation_rejects_empty_long_and_unsafe_comments() {
        assert!(matches!(
            comment_body("  ".to_string()),
            Err(ServerError::Validation(_))
        ));
        assert!(matches!(
            comment_body("a".repeat(2001)),
            Err(ServerError::Validation(_))
        ));
        assert!(matches!(
            comment_body("<script>alert(1)</script>".to_string()),
            Err(ServerError::Validation(_))
        ));
    }

    #[test]
    fn validation_trims_and_allows_clean_mild_and_moderate_messages() {
        for message in [
            "This is a clean message",
            "This is a bad word: crap",
            "F u c k",
        ] {
            let payload = guestbook_entry(request(&format!("  {message}  "))).unwrap();
            assert_eq!(payload.message, message);
        }
    }

    #[test]
    fn validation_rejects_severe_messages_with_user_facing_error() {
        assert_eq!(
            guestbook_entry(request("  i hope you die  ")),
            Err(ServerError::Validation(
                "Message contains offensive content".to_string()
            ))
        );
    }

    #[test]
    fn validation_preserves_message_length_limits() {
        for message in [String::new(), "  ".to_string(), "a".repeat(256)] {
            assert_eq!(
                guestbook_entry(request(&message)),
                Err(ServerError::Validation(
                    "Message must be between 1 and 255 characters".to_string()
                ))
            );
        }
        assert!(guestbook_entry(request("a")).is_ok());
        assert!(guestbook_entry(request(&format!("{}hello", "café ".repeat(50)))).is_ok());
    }

    #[test]
    fn validation_accepts_png_signatures_and_rejects_anything_else() {
        let signed = |bytes: &[u8]| CreateEntryRequest {
            signature: Some(STANDARD.encode(bytes)),
            ..request("hello")
        };
        let png = b"\x89PNG\r\n\x1a\nrest";
        let oversized = [png.as_slice(), &vec![0; MAX_SIGNATURE_BYTES]].concat();
        let rejected = Err(ServerError::Validation(
            "Signature must be a PNG drawing under 256 KB".to_string(),
        ));

        assert!(guestbook_entry(signed(png)).is_ok());
        assert_eq!(guestbook_entry(signed(b"<svg/>")), rejected);
        assert_eq!(guestbook_entry(signed(&oversized)), rejected);
        assert_eq!(
            guestbook_entry(CreateEntryRequest {
                signature: Some("not base64!".into()),
                ..request("hello")
            }),
            rejected
        );
    }
}

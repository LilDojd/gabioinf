#[cfg(feature = "server")]
use crate::backend::{AppState, auth::SessionWrapper};
use crate::shared::{
    models::{Comment, CommentId},
    server_fns::ServerError,
};
use dioxus::prelude::*;

#[cfg(feature = "server")]
fn validate_body(body: String) -> Result<(String, String), ServerError> {
    let body = body.trim().to_string();
    if !(1..=2000).contains(&body.chars().count()) {
        return Err(ServerError::Validation(
            "Comment must be between 1 and 2000 characters".to_string(),
        ));
    }
    crate::backend::profanity::validate_no_severe_content(&body)
        .map_err(|_| ServerError::Validation("Comment contains offensive content".to_string()))?;
    let body_html = crate::backend::markdown::render(&body)
        .map_err(|error| ServerError::Validation(format!("Invalid Markdown: {error}")))?;
    Ok((body, body_html))
}

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn post_comment(
    slug: String,
    body: String,
    parent_id: Option<CommentId>,
) -> Result<Comment, ServerError> {
    let user = session.session.user.ok_or(ServerError::Unauthenticated)?;
    if crate::blog::find_post(&slug).is_none() {
        return Err(ServerError::Validation(
            "That blog post does not exist".to_string(),
        ));
    }
    let (body, body_html) = validate_body(body)?;
    let row = state
        .comment_repo
        .create(&slug, user.id, parent_id, &body)
        .await
        .map_err(|error| ServerError::internal("create comment", error))?
        .ok_or_else(|| {
            ServerError::Validation(
                "Reply target must be a top-level comment on this post".to_string(),
            )
        })?;

    Ok(super::comment_with_html(row, body_html))
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn validation_trims_and_renders_safe_markdown() {
        let (body, html) = validate_body("  **hello**  ".to_string()).unwrap();

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
            let (body, html) = validate_body(format!("  {text}  ")).unwrap();
            assert_eq!(body, text);
            assert_eq!(html, format!("<p>{text}</p>\n"));
        }
    }

    #[test]
    fn validation_rejects_severe_comments_with_user_facing_error() {
        assert_eq!(
            validate_body("  i hope you die  ".to_string()),
            Err(ServerError::Validation(
                "Comment contains offensive content".to_string()
            ))
        );
    }

    #[test]
    fn validation_rejects_empty_long_and_unsafe_comments() {
        assert!(matches!(
            validate_body("  ".to_string()),
            Err(ServerError::Validation(_))
        ));
        assert!(matches!(
            validate_body("a".repeat(2001)),
            Err(ServerError::Validation(_))
        ));
        assert!(matches!(
            validate_body("<script>alert(1)</script>".to_string()),
            Err(ServerError::Validation(_))
        ));
    }
}

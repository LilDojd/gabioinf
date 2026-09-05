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
    let (body_html, visible_text) = crate::backend::markdown::render_with_text(&body)
        .map_err(|error| ServerError::Validation(format!("Invalid Markdown: {error}")))?;
    crate::backend::profanity::validate_no_severe_content(&visible_text)
        .map_err(|_| ServerError::Validation("Comment contains offensive content".to_string()))?;
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
    use crate::shared::server_fns::moderation_test_context;

    #[sqlx::test]
    async fn authenticated_comment_rejects_severe_content_without_inserting(pool: sqlx::PgPool) {
        let context = moderation_test_context(pool.clone()).await;
        let slug = crate::blog::published_posts()
            .next()
            .expect("published post fixture")
            .slug;
        context
            .scope(async {
                for body in [
                    "i hope you die",
                    "i h&#111;p&#101; y&#111;u d&#105;&#101;",
                    "i ho[pe](https://example.com) you die",
                ] {
                    assert_eq!(
                        post_comment(slug.into(), body.into(), None).await,
                        Err(ServerError::Validation(
                            "Comment contains offensive content".into()
                        )),
                        "{body:?}"
                    );
                    assert_eq!(
                        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM comments")
                            .fetch_one(&pool)
                            .await
                            .unwrap(),
                        0
                    );
                }

                let comment = post_comment(slug.into(), "  **hello**  ".into(), None)
                    .await
                    .unwrap();
                let stored: String = sqlx::query_scalar("SELECT body FROM comments WHERE id = $1")
                    .bind(comment.id.0)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
                assert_eq!(stored, "**hello**");
                assert!(comment.body_html.contains("<strong>hello</strong>"));
            })
            .await;
    }

    #[sqlx::test]
    async fn authenticated_reply_uses_the_same_moderation_policy(pool: sqlx::PgPool) {
        let context = moderation_test_context(pool.clone()).await;
        let slug = crate::blog::published_posts()
            .next()
            .expect("published post fixture")
            .slug;
        context
            .scope(async {
                let root = post_comment(slug.into(), "hello".into(), None)
                    .await
                    .unwrap();
                assert_eq!(
                    post_comment(
                        slug.into(),
                        "i ho[pe](https://example.com) you die".into(),
                        Some(root.id)
                    )
                    .await,
                    Err(ServerError::Validation(
                        "Comment contains offensive content".into()
                    ))
                );
                assert_eq!(
                    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM comments")
                        .fetch_one(&pool)
                        .await
                        .unwrap(),
                    1
                );

                let reply = post_comment(slug.into(), "F u c k".into(), Some(root.id))
                    .await
                    .unwrap();
                let stored: (String, Option<i64>) =
                    sqlx::query_as("SELECT body, parent_id FROM comments WHERE id = $1")
                        .bind(reply.id.0)
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                assert_eq!(stored, ("F u c k".into(), Some(root.id.0)));
            })
            .await;
    }

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
    fn validation_rejects_equivalent_severe_text_in_entities_and_links() {
        for body in [
            "i h&#111;p&#101; y&#111;u d&#105;&#101;",
            "i h&#x6f;p&#x65; y&#x6f;u d&#x69;&#x65;",
            "i ho[pe](https://example.com) you die",
            "i ho[**pe**](https://example.com) you die",
        ] {
            assert_eq!(
                validate_body(body.to_string()),
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
            validate_body("[hello](https://example.com \"i hope you die\")".to_string()),
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
                validate_body(body.to_string()).unwrap_or_else(|error| panic!("{body:?}: {error}"));
            assert_eq!(stored, body);
        }
    }

    #[test]
    fn validation_keeps_encoded_html_as_escaped_text() {
        let (_, html) = validate_body("&lt;script&gt;alert(1)&lt;/script&gt;".to_string()).unwrap();
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn validation_rejects_unsafe_markdown_links() {
        assert!(matches!(
            validate_body("[click](javascript:alert(1))".to_string()),
            Err(ServerError::Validation(message)) if message.starts_with("Invalid Markdown:")
        ));
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

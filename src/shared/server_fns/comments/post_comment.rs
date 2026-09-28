#[cfg(feature = "server")]
use crate::backend::{AppState, auth::AuthSession};
use crate::shared::{
    models::{Comment, CommentId},
    server_fns::ServerError,
};
use dioxus::prelude::*;

#[server(auth:AuthSession, state:axum::Extension<AppState>)]
pub async fn post_comment(
    slug: String,
    body: String,
    parent_id: Option<CommentId>,
) -> Result<Comment, ServerError> {
    let user = auth.user().await.ok_or(ServerError::Unauthenticated)?;
    if crate::blog::find_post(&slug).is_none() {
        return Err(ServerError::Validation(
            "That blog post does not exist".to_string(),
        ));
    }
    let (body, body_html) = crate::backend::validation::comment_body(body)?;
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

    Ok(row.with_html(body_html))
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use crate::backend::test_support::moderation_test_context;

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
}

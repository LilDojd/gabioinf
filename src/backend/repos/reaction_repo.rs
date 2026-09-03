use crate::{
    backend::errors::{ApiError, BResult},
    shared::models::{CommentId, Emoji, GuestId, ReactionCount, ReactionTarget, Reactions},
};

/// Mirrors PostgreSQL's enum while the shared target also carries its identifier.
#[derive(Clone, Copy, Debug, sqlx::Type)]
#[sqlx(type_name = "reaction_target", rename_all = "lowercase")]
enum ReactionTargetKind {
    Post,
    Comment,
}

#[derive(Clone, Debug)]
pub struct ReactionRepo {
    pool: sqlx::PgPool,
}

impl ReactionRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn counts_for_post(&self, slug: &str, viewer: Option<GuestId>) -> BResult<Reactions> {
        let viewer = viewer.map(|id| id.as_value());
        let rows = sqlx::query!(
            r#"
            SELECT
                comment_id AS "comment_id?: CommentId",
                emoji,
                COUNT(*) AS "count!",
                COALESCE(bool_or(guest_id = $2::BIGINT), false) AS "reacted!"
            FROM reactions
            WHERE post_slug = $1
            GROUP BY comment_id, emoji
            "#,
            slug,
            viewer,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut reactions = Reactions::default();
        for row in rows {
            let count = ReactionCount {
                emoji: row
                    .emoji
                    .parse()
                    .map_err(|_| ApiError::InvalidData("unknown reaction emoji"))?,
                count: u32::try_from(row.count)
                    .map_err(|_| ApiError::InvalidData("reaction count exceeds u32"))?,
                reacted: row.reacted,
            };
            if let Some(comment_id) = row.comment_id {
                reactions
                    .comments
                    .entry(comment_id)
                    .or_default()
                    .push(count);
            } else {
                reactions.post.push(count);
            }
        }
        let order =
            |count: &ReactionCount| Emoji::ALL.iter().position(|emoji| *emoji == count.emoji);
        reactions.post.sort_by_key(order);
        for counts in reactions.comments.values_mut() {
            counts.sort_by_key(order);
        }
        Ok(reactions)
    }

    /// Toggles one typed reaction and returns whether it is now present.
    pub async fn toggle(
        &self,
        target: ReactionTarget,
        guest_id: GuestId,
        emoji: Emoji,
    ) -> BResult<bool> {
        let mut transaction = self.pool.begin().await?;
        let (target_kind, post_slug, comment_id) = match target {
            ReactionTarget::Post { slug } => (ReactionTargetKind::Post, slug, None),
            ReactionTarget::Comment(id) => {
                let Some(post_slug) =
                    sqlx::query_scalar!("SELECT post_slug FROM comments WHERE id = $1", id.0,)
                        .fetch_optional(&mut *transaction)
                        .await?
                else {
                    return Ok(false);
                };
                (ReactionTargetKind::Comment, post_slug, Some(id.0))
            }
        };
        let inserted = sqlx::query!(
            r#"
            INSERT INTO reactions (target_kind, post_slug, comment_id, guest_id, emoji)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT DO NOTHING
            "#,
            target_kind as ReactionTargetKind,
            post_slug,
            comment_id,
            guest_id.as_value(),
            emoji.name(),
        )
        .execute(&mut *transaction)
        .await?
        .rows_affected()
            == 1;

        if !inserted {
            sqlx::query!(
                r#"
                DELETE FROM reactions
                WHERE target_kind = $1
                  AND post_slug = $2
                  AND comment_id IS NOT DISTINCT FROM $3
                  AND guest_id = $4
                  AND emoji = $5
                "#,
                target_kind as ReactionTargetKind,
                post_slug,
                comment_id,
                guest_id.as_value(),
                emoji.name(),
            )
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(inserted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::repos::{CommentRepo, GuestRepo},
        shared::models::{GithubId, Guest},
    };
    use sqlx::PgPool;

    async fn create_guest(pool: &PgPool, number: i64) -> Guest {
        GuestRepo::new(pool.clone())
            .upsert(&Guest {
                github_id: GithubId(number),
                name: format!("Test User {number}"),
                username: format!("reactionuser{number}"),
                ..Default::default()
            })
            .await
            .unwrap()
    }

    #[sqlx::test]
    async fn toggle_updates_counts_and_viewer_flag(pool: PgPool) {
        let first = create_guest(&pool, 1).await;
        let second = create_guest(&pool, 2).await;
        let repo = ReactionRepo::new(pool);
        let target = || ReactionTarget::Post {
            slug: "post".to_string(),
        };

        assert!(repo.toggle(target(), first.id, Emoji::Alien).await.unwrap());
        assert!(
            repo.toggle(target(), second.id, Emoji::Alien)
                .await
                .unwrap()
        );
        assert!(repo.toggle(target(), second.id, Emoji::Fire).await.unwrap());
        assert_eq!(
            repo.counts_for_post("post", Some(first.id))
                .await
                .unwrap()
                .post,
            vec![
                ReactionCount {
                    emoji: Emoji::Alien,
                    count: 2,
                    reacted: true,
                },
                ReactionCount {
                    emoji: Emoji::Fire,
                    count: 1,
                    reacted: false,
                },
            ]
        );

        assert!(!repo.toggle(target(), first.id, Emoji::Alien).await.unwrap());
        assert_eq!(
            repo.counts_for_post("post", Some(first.id))
                .await
                .unwrap()
                .post,
            vec![
                ReactionCount {
                    emoji: Emoji::Alien,
                    count: 1,
                    reacted: false,
                },
                ReactionCount {
                    emoji: Emoji::Fire,
                    count: 1,
                    reacted: false,
                },
            ]
        );
    }

    #[sqlx::test]
    async fn deleting_a_comment_cascades_its_reactions(pool: PgPool) {
        let guest = create_guest(&pool, 1).await;
        let comments = CommentRepo::new(pool.clone());
        let comment = comments
            .create("post", guest.id, None, "Comment")
            .await
            .unwrap()
            .unwrap();
        let reactions = ReactionRepo::new(pool);

        assert!(
            reactions
                .toggle(ReactionTarget::Comment(comment.id), guest.id, Emoji::Heart)
                .await
                .unwrap()
        );
        assert!(
            reactions
                .counts_for_post("post", None)
                .await
                .unwrap()
                .comments
                .contains_key(&comment.id)
        );

        assert!(comments.delete_owned(comment.id, guest.id).await.unwrap());
        assert!(
            reactions
                .counts_for_post("post", None)
                .await
                .unwrap()
                .comments
                .is_empty()
        );
    }
}

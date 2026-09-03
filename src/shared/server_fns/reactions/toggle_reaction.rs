#[cfg(feature = "server")]
use crate::backend::{
    AppState,
    auth::SessionWrapper,
    repos::{CommentRepo, ReactionRepo},
};
#[cfg(feature = "server")]
use crate::shared::models::GuestId;
use crate::shared::{
    models::{Emoji, ReactionCount, ReactionTarget},
    server_fns::ServerError,
};
use dioxus::prelude::*;

#[server(session:SessionWrapper, state:axum::Extension<AppState>)]
pub async fn toggle_reaction(
    target: ReactionTarget,
    emoji: Emoji,
) -> Result<Vec<ReactionCount>, ServerError> {
    let user = session.session.user.ok_or(ServerError::Unauthenticated)?;
    toggle_for_guest(
        &state.reaction_repo,
        &state.comment_repo,
        target,
        user.id,
        emoji,
    )
    .await
}

#[cfg(feature = "server")]
async fn toggle_for_guest(
    reaction_repo: &ReactionRepo,
    comment_repo: &CommentRepo,
    target: ReactionTarget,
    guest_id: GuestId,
    emoji: Emoji,
) -> Result<Vec<ReactionCount>, ServerError> {
    let (slug, comment_id) = match &target {
        ReactionTarget::Post { slug } => {
            if crate::blog::find_post(slug).is_none() {
                return Err(ServerError::NotFound);
            }
            (slug.clone(), None)
        }
        ReactionTarget::Comment(id) => {
            let slug = comment_repo
                .post_slug(*id)
                .await
                .map_err(|error| ServerError::internal("find reaction comment", error))?
                .ok_or(ServerError::NotFound)?;
            (slug, Some(*id))
        }
    };

    reaction_repo
        .toggle(target, guest_id, emoji)
        .await
        .map_err(|error| ServerError::internal("toggle reaction", error))?;
    let mut reactions = reaction_repo
        .counts_for_post(&slug, Some(guest_id))
        .await
        .map_err(|error| ServerError::internal("reload reactions", error))?;
    Ok(match comment_id {
        Some(id) => reactions.comments.remove(&id).unwrap_or_default(),
        None => reactions.post,
    })
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use crate::shared::models::CommentId;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn unknown_comment_is_not_found(pool: PgPool) {
        let result = toggle_for_guest(
            &ReactionRepo::new(pool.clone()),
            &CommentRepo::new(pool),
            ReactionTarget::Comment(CommentId(999)),
            GuestId(1),
            Emoji::Alien,
        )
        .await;

        assert_eq!(result, Err(ServerError::NotFound));
    }
}

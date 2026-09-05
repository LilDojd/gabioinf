use crate::{
    components::{GithubMark, ReactionBar, server_error_message},
    shared::{
        models::{
            Comment, CommentAuthor, CommentId, Guest, ReactionCount, ReactionTarget, Reactions,
        },
        server_fns::{self, ServerError},
    },
};
use dioxus::prelude::*;
use time::macros::format_description;

const COMMENT_DATE: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[day padding:none] [month repr:short] [year]");

#[component]
pub fn Comments(
    slug: &'static str,
    mut reactions: dioxus::fullstack::Loader<Reactions>,
    mut viewer: dioxus::fullstack::Loader<Option<Guest>>,
) -> Element {
    let mut comments = use_loader(move || server_fns::load_comments(slug.to_string()))?;
    let mut body = use_signal(String::new);
    let mut reply_target = use_signal(|| None::<CommentId>);
    let mut submitting = use_signal(|| false);
    let mut submit_error = use_signal(|| None::<String>);
    let action_error = use_signal(|| None::<(CommentId, String)>);
    let deleting = use_signal(|| false);

    let reply_author = reply_target().and_then(|id| {
        comments
            .read()
            .iter()
            .find(|comment| comment.id == id)
            .map(|comment| comment.author.username.clone())
    });

    rsx! {
        section {
            class: "mt-8 flex flex-col gap-6 border-t border-line pt-7",
            aria_labelledby: "comments-heading",
            header { class: "flex items-baseline justify-between gap-4",
                h2 { id: "comments-heading", class: "label-mono m-0", "// comments · {comments.read().len()}" }
                match viewer.read().as_ref() {
                    Some(guest) => rsx! {
                        span { class: "text-[13px] text-muted",
                            "signed in as {guest.username} · "
                            button {
                                r#type: "button",
                                class: "bg-transparent p-0 text-muted hover:text-accent",
                                onclick: move |_| {
                                    spawn(async move {
                                        match server_fns::logout().await {
                                            Ok(()) => viewer.set(None),
                                            Err(error) => {
                                                tracing::error!("Could not sign out: {error:?}");
                                                submit_error.set(Some(server_error_message(&error, "Could not sign out. Please retry.")));
                                            }
                                        }
                                    });
                                },
                                "sign out"
                            }
                        }
                    },
                    None => rsx! {
                        a {
                            href: "/v1/login?next=/blog/{slug}",
                            class: "inline-flex items-center gap-2 text-[13px] text-muted no-underline casual hover:text-accent",
                            GithubMark { size: 14 }
                            "sign in to comment"
                        }
                    },
                }
            }

            if viewer.read().is_some() {
                form {
                    class: "flex flex-col gap-2",
                    onsubmit: move |event| {
                        event.prevent_default();
                        if submitting() {
                            return;
                        }
                        submitting.set(true);
                        submit_error.set(None);
                        let comment_body = body();
                        let parent_id = reply_target();
                        spawn(async move {
                            match server_fns::post_comment(slug.to_string(), comment_body, parent_id).await {
                                Ok(comment) => {
                                    comments.write().push(comment);
                                    body.set(String::new());
                                    reply_target.set(None);
                                }
                                Err(error) => {
                                    tracing::error!("Could not post comment: {error:?}");
                                    submit_error.set(Some(server_error_message(&error, "Could not post your comment. Please retry.")));
                                }
                            }
                            submitting.set(false);
                        });
                    },
                    if let Some(username) = reply_author {
                        span { class: "label-mono",
                            "replying to @{username} · "
                            button {
                                r#type: "button",
                                class: "bg-transparent p-0 text-label hover:text-accent",
                                onclick: move |_| reply_target.set(None),
                                "cancel"
                            }
                        }
                    }
                    textarea {
                        class: "prose-font w-full resize-y rounded-md border border-card bg-surface px-3.5 py-3 text-base leading-[1.45] text-text outline-none placeholder:text-label focus:border-accent",
                        placeholder: "say something. markdown works.",
                        rows: 3,
                        value: body,
                        disabled: submitting(),
                        oninput: move |event| body.set(event.value()),
                    }
                    if let Some(error) = submit_error.read().as_ref() {
                        span { role: "alert", class: "label-mono text-mars", {error.to_string()} }
                    }
                    div { class: "flex items-center justify-between gap-3",
                        span { class: "label-mono text-faint", "be kind. no trackers, no paywall, just a database row." }
                        button { r#type: "submit", class: "btn-primary shrink-0 px-3.5 py-[7px] text-[13px]", disabled: submitting(),
                            if submitting() { "posting…" } else { "post" }
                        }
                    }
                }
            }

            div { class: "flex flex-col",
                for root in comments.read().iter().filter(|comment| comment.parent_id.is_none()) {
                    CommentRow {
                        key: "{root.id.0}",
                        comment: root.clone(),
                        viewer: viewer.read().clone(),
                        can_reply: viewer.read().is_some(),
                        reaction_counts: reactions.read().comments.get(&root.id).cloned().unwrap_or_default(),
                        error: action_error.read().as_ref().filter(|(id, _)| *id == root.id).map(|(_, error)| error.clone()),
                        on_reply: move |id| reply_target.set(Some(id)),
                        deleting: deleting(),
                        on_delete: move |id| request_delete(comments, action_error, deleting, id),
                        on_reactions_change: {
                            let id = root.id;
                            move |counts| { reactions.write().comments.insert(id, counts); }
                        },
                    }
                    for reply in comments.read().iter().filter(|comment| comment.parent_id == Some(root.id)) {
                        div { key: "{reply.id.0}", class: "ml-[42px]",
                            CommentRow {
                                comment: reply.clone(),
                                viewer: viewer.read().clone(),
                                can_reply: false,
                                reaction_counts: reactions.read().comments.get(&reply.id).cloned().unwrap_or_default(),
                                error: action_error.read().as_ref().filter(|(id, _)| *id == reply.id).map(|(_, error)| error.clone()),
                                on_reply: move |_| {},
                                deleting: deleting(),
                                on_delete: move |id| request_delete(comments, action_error, deleting, id),
                                on_reactions_change: {
                                    let id = reply.id;
                                    move |counts| { reactions.write().comments.insert(id, counts); }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommentRow(
    comment: Comment,
    viewer: Option<Guest>,
    can_reply: bool,
    deleting: bool,
    reaction_counts: Vec<ReactionCount>,
    error: Option<String>,
    on_reply: EventHandler<CommentId>,
    on_delete: EventHandler<CommentId>,
    on_reactions_change: EventHandler<Vec<ReactionCount>>,
) -> Element {
    let author = &comment.author;
    let own_comment = viewer
        .as_ref()
        .is_some_and(|guest| guest.github_id == author.github_id);
    let avatar = format!(
        "https://avatars.githubusercontent.com/u/{}?s=56",
        author.github_id.as_value()
    );
    let profile = format!("https://github.com/{}", author.username);

    rsx! {
        article { class: "grid grid-cols-[28px_1fr] gap-3.5 border-t border-line py-[18px]",
            img { class: "mt-0.5 size-7 rounded-full", src: avatar, loading: "lazy", alt: "" }
            div { class: "flex min-w-0 flex-col gap-1.5",
                CommentMeta { author: author.clone(), profile, created_at: comment.created_at }
                div { class: "comment-body", dangerous_inner_html: comment.body_html }
                ReactionBar {
                    target: ReactionTarget::Comment(comment.id),
                    counts: reaction_counts,
                    signed_in: viewer.is_some(),
                    on_change: on_reactions_change,
                }
                div { class: "flex gap-3",
                    if can_reply {
                        button { r#type: "button", class: "comment-action", onclick: move |_| on_reply.call(comment.id), "reply" }
                    }
                    if own_comment {
                        button { r#type: "button", class: "comment-action", disabled: deleting, onclick: move |_| on_delete.call(comment.id), "delete" }
                    }
                }
                if let Some(error) = error {
                    span { role: "alert", class: "label-mono text-mars", {error} }
                }
            }
        }
    }
}

#[component]
fn CommentMeta(
    author: CommentAuthor,
    profile: String,
    created_at: time::OffsetDateTime,
) -> Element {
    rsx! {
        div { class: "label-mono flex items-baseline gap-2.5",
            a { href: profile, target: "_blank", rel: "noopener noreferrer", class: "text-secondary no-underline hover:text-accent", "{author.username}" }
            time { datetime: created_at.to_string(), "{comment_date(created_at)}" }
            if author.is_owner {
                span { class: "rounded-[3px] bg-[rgb(194_249_187_/_0.1)] px-1.5 py-0.5 text-[10px] text-accent", "author" }
            }
        }
    }
}

fn comment_date(date: time::OffsetDateTime) -> String {
    date.format(COMMENT_DATE)
        .expect("the static comment date format is valid")
        .to_lowercase()
}

fn request_delete(
    mut comments: dioxus::fullstack::Loader<Vec<Comment>>,
    mut action_error: Signal<Option<(CommentId, String)>>,
    mut deleting: Signal<bool>,
    id: CommentId,
) {
    if deleting() {
        return;
    }
    deleting.set(true);
    action_error.set(None);
    spawn(async move {
        let result = server_fns::delete_comment(id).await;
        if let Err(error) = apply_deletion_result(&mut comments.write(), id, result) {
            tracing::error!("Could not delete comment: {error:?}");
            action_error.set(Some((
                id,
                server_error_message(&error, "Could not delete your comment. Please retry."),
            )));
        }
        deleting.set(false);
    });
}

fn apply_deletion_result(
    comments: &mut Vec<Comment>,
    id: CommentId,
    result: Result<(), ServerError>,
) -> Result<(), ServerError> {
    result?;
    comments.retain(|comment| comment.id != id && comment.parent_id != Some(id));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::models::OWNER_GITHUB_ID;
    use time::macros::datetime;

    fn comment(id: i64, parent_id: Option<CommentId>) -> Comment {
        Comment {
            id: CommentId(id),
            parent_id,
            author: CommentAuthor {
                username: "LilDojd".to_string(),
                github_id: OWNER_GITHUB_ID,
                is_owner: true,
            },
            body_html: "<p>hello</p>".to_string(),
            created_at: datetime!(2026-06-14 0:00 UTC),
        }
    }

    #[test]
    fn deleting_root_removes_its_replies_but_not_other_threads() {
        let mut comments = vec![
            comment(1, None),
            comment(2, Some(CommentId(1))),
            comment(3, None),
        ];

        apply_deletion_result(&mut comments, CommentId(1), Ok(())).unwrap();

        assert_eq!(comments, vec![comment(3, None)]);
    }

    #[test]
    fn deletion_results_preserve_updates_made_while_the_request_was_pending() {
        for result in [Ok(()), Err(ServerError::Unavailable)] {
            let mut comments = vec![comment(1, None), comment(2, None)];
            let pending_id = CommentId(1);

            // Another update removes a row and adds a comment before deletion completes.
            comments.retain(|comment| comment.id != CommentId(2));
            comments.push(comment(3, None));

            assert_eq!(
                apply_deletion_result(&mut comments, pending_id, result.clone()),
                result
            );
            let expected = if result.is_ok() {
                vec![comment(3, None)]
            } else {
                vec![comment(1, None), comment(3, None)]
            };
            assert_eq!(comments, expected);
        }
    }
}

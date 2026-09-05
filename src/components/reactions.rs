use crate::{
    Route,
    components::server_error_message,
    shared::{
        models::{Emoji, ReactionCount, ReactionTarget},
        server_fns,
    },
};
use dioxus::prelude::*;

#[component]
pub fn ReactionBar(
    target: ReactionTarget,
    counts: Vec<ReactionCount>,
    signed_in: bool,
    on_change: EventHandler<Vec<ReactionCount>>,
) -> Element {
    let mut ui = ReactionUi {
        picker_open: use_signal(|| false),
        sign_in_hint: use_signal(|| false),
        pending: use_signal(|| false),
        error: use_signal(|| None::<String>),
    };
    let next = match use_route::<Route>() {
        Route::BlogPost { slug } => format!("/blog/{slug}"),
        _ => "/blog".to_string(),
    };

    rsx! {
        div { class: "reaction-bar",
            for count in counts.iter().filter(|count| count.count > 0 || count.reacted).cloned() {
                button {
                    key: "{count.emoji.name()}",
                    r#type: "button",
                    class: if count.reacted { "reaction-chip reaction-chip-active" } else { "reaction-chip" },
                    aria_label: "{count.emoji.name()} reaction, {count.count}",
                    aria_pressed: count.reacted,
                    disabled: (ui.pending)(),
                    title: count.emoji.name(),
                    onclick: {
                        let target = target.clone();
                        let counts = counts.clone();
                        move |_| request_reaction(
                            target.clone(),
                            count.emoji,
                            counts.clone(),
                            signed_in,
                            ui,
                            on_change,
                        )
                    },
                    span { class: "reaction-emoji", aria_hidden: "true", "{count.emoji.glyph()}" }
                    span { class: "reaction-count", "{count.count}" }
                }
            }
            button {
                r#type: "button",
                class: "reaction-chip reaction-add",
                aria_label: "add reaction",
                aria_expanded: (ui.picker_open)(),
                onclick: move |_| {
                    ui.sign_in_hint.set(false);
                    ui.picker_open.toggle();
                },
                "+"
            }
            if (ui.picker_open)() {
                button {
                    r#type: "button",
                    class: "reaction-backdrop",
                    aria_label: "close reaction picker",
                    onclick: move |_| ui.picker_open.set(false),
                }
                div { class: "reaction-popover", role: "dialog", aria_label: "Choose a reaction",
                    for emoji in Emoji::ALL {
                        button {
                            key: "{emoji.name()}",
                            r#type: "button",
                            title: emoji.name(),
                            aria_label: emoji.name(),
                            disabled: (ui.pending)(),
                            onclick: {
                                let target = target.clone();
                                let counts = counts.clone();
                                move |_| request_reaction(
                                    target.clone(),
                                    emoji,
                                    counts.clone(),
                                    signed_in,
                                    ui,
                                    on_change,
                                )
                            },
                            "{emoji.glyph()}"
                        }
                    }
                }
            }
            if (ui.sign_in_hint)() {
                span { class: "reaction-hint",
                    a { href: "/v1/login?next={next}", "sign in to react" }
                }
            }
            if let Some(message) = ui.error.read().as_ref() {
                span { role: "alert", class: "reaction-error", {message.to_string()} }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ReactionUi {
    picker_open: Signal<bool>,
    sign_in_hint: Signal<bool>,
    pending: Signal<bool>,
    error: Signal<Option<String>>,
}

fn request_reaction(
    target: ReactionTarget,
    emoji: Emoji,
    counts: Vec<ReactionCount>,
    signed_in: bool,
    mut ui: ReactionUi,
    on_change: EventHandler<Vec<ReactionCount>>,
) {
    ui.picker_open.set(false);
    ui.error.set(None);
    if !signed_in {
        ui.sign_in_hint.set(true);
        return;
    }
    if (ui.pending)() {
        return;
    }

    ui.sign_in_hint.set(false);
    ui.pending.set(true);
    let previous = counts;
    on_change.call(optimistic_counts(previous.clone(), emoji));
    spawn(async move {
        match server_fns::toggle_reaction(target, emoji).await {
            Ok(fresh) => on_change.call(fresh),
            Err(server_error) => {
                tracing::error!("Could not toggle reaction: {server_error:?}");
                on_change.call(previous);
                ui.error.set(Some(server_error_message(
                    &server_error,
                    "Could not save your reaction. Please retry.",
                )));
            }
        }
        ui.pending.set(false);
    });
}

fn optimistic_counts(mut counts: Vec<ReactionCount>, emoji: Emoji) -> Vec<ReactionCount> {
    if let Some(count) = counts.iter_mut().find(|count| count.emoji == emoji) {
        if count.reacted {
            count.count = count.count.saturating_sub(1);
        } else {
            count.count = count.count.saturating_add(1);
        }
        count.reacted = !count.reacted;
    } else {
        counts.push(ReactionCount {
            emoji,
            count: 1,
            reacted: true,
        });
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimistic_toggle_updates_count_and_viewer_state() {
        let counts = vec![ReactionCount {
            emoji: Emoji::Alien,
            count: 2,
            reacted: false,
        }];

        let added = optimistic_counts(counts, Emoji::Alien);
        assert_eq!(added[0].count, 3);
        assert!(added[0].reacted);

        let removed = optimistic_counts(added, Emoji::Alien);
        assert_eq!(removed[0].count, 2);
        assert!(!removed[0].reacted);
    }
}

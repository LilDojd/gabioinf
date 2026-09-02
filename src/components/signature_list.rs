use crate::auth::AuthState;
use crate::components::{ButtonVariant, Card, CardType, CloseButton, Loading, StyledButton};
use crate::shared::{
    models::{GuestbookEntry, GuestbookId},
    server_fns,
};
use dioxus::prelude::*;

const SIGNATURES_PER_PAGE: usize = 10;

#[component]
pub fn SignatureList() -> Element {
    let mut auth_state = use_context::<dioxus::fullstack::Loader<AuthState>>();
    let mut entries = use_signal(Vec::<GuestbookEntry>::new);
    let mut next_cursor = use_signal(|| None);
    let mut loaded_once = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut load_error = use_signal(|| None::<String>);
    let mut deleting = use_signal(|| false);
    let mut delete_error = use_signal(|| None::<String>);
    let mut deleted_entry_id = use_signal(|| None::<GuestbookId>);

    let load_more = use_callback(move |_| {
        if loading() || (loaded_once() && next_cursor().is_none()) {
            return;
        }
        let cursor = next_cursor();
        loading.set(true);
        load_error.set(None);
        spawn(async move {
            match server_fns::load_signatures(cursor).await {
                Ok(page) => {
                    append_unique(&mut entries.write(), page.entries, deleted_entry_id());
                    next_cursor.set(page.next_cursor);
                    loaded_once.set(true);
                }
                Err(error) => {
                    dioxus_logger::tracing::error!("Could not load signatures: {error:?}");
                    load_error.set(Some(
                        "Could not load signatures. Check your connection and retry.".to_string(),
                    ));
                }
            }
            loading.set(false);
        });
    });

    use_effect(move || {
        if !loaded_once() && !loading() && load_error.read().is_none() {
            load_more.call(());
        }
    });

    let user_entry = match &*auth_state.read() {
        AuthState::Authenticated(user_state) => user_state.entry.clone(),
        _ => None,
    };
    let user_entry_id = user_entry.as_ref().map(|entry| entry.id);

    rsx! {
        if let Some(error) = delete_error.read().as_ref() {
            div {
                role: "alert",
                class: "mb-4 flex items-center gap-3 text-coral",
                span { {error.clone()} }
            }
        }
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
            if let Some(user_entry) = user_entry {
                {
                    let id = user_entry.id;
                    rsx! {
                        Card {
                            card_type: CardType::Signature {
                                entry: user_entry,
                                close_button: rsx! {
                                    CloseButton {
                                        layout: "absolute top-2 right-2 w-6 h-6",
                                        disabled: deleting(),
                                        onclick: move |_| {
                                            if deleting() {
                                                return;
                                            }
                                            deleting.set(true);
                                            delete_error.set(None);
                                            spawn(async move {
                                                match server_fns::delete_signature(id).await {
                                                    Ok(()) => {
                                                        deleted_entry_id.set(Some(id));
                                                        entries.write().retain(|entry| entry.id != id);
                                                        if let AuthState::Authenticated(user_state) =
                                                            &mut *auth_state.write()
                                                        {
                                                            user_state.entry = None;
                                                        }
                                                    }
                                                    Err(error) => {
                                                        dioxus_logger::tracing::error!(
                                                            "Error deleting signature: {error:?}"
                                                        );
                                                        delete_error.set(Some(
                                                            "Could not delete your signature. Retry with the × button."
                                                                .to_string(),
                                                        ));
                                                    }
                                                }
                                                deleting.set(false);
                                            });
                                        },
                                    }
                                    if deleting() {
                                        span {
                                            class: "absolute top-3 right-10 text-xs text-stone-400",
                                            "Deleting…"
                                        }
                                    }
                                },
                            },
                        }
                    }
                }
            }
            for entry in entries.read().iter().filter(|entry| Some(entry.id) != user_entry_id) {
                Card {
                    key: "{entry.id.as_value()}",
                    card_type: CardType::Signature {
                        entry: entry.clone(),
                        close_button: rsx! {},
                    },
                }
            }
            if loading() && !loaded_once() {
                for _ in 0..SIGNATURES_PER_PAGE {
                    Card { card_type: CardType::Skeleton }
                }
            }
        }
        if loading() && loaded_once() {
            Loading {}
        } else if let Some(error) = load_error.read().as_ref() {
            div {
                role: "alert",
                class: "mt-6 flex flex-col items-center gap-3 text-coral",
                span { {error.clone()} }
                StyledButton {
                    text: "Retry",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| load_more.call(()),
                }
            }
        } else if loaded_once() && next_cursor.read().is_some() {
            div { class: "mt-6 flex justify-center",
                StyledButton {
                    text: "Load more",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| load_more.call(()),
                }
            }
        }
    }
}

fn append_unique(
    entries: &mut Vec<GuestbookEntry>,
    incoming: Vec<GuestbookEntry>,
    hidden: Option<GuestbookId>,
) {
    for entry in incoming {
        if Some(entry.id) != hidden && !entries.iter().any(|current| current.id == entry.id) {
            entries.push(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: i64) -> GuestbookEntry {
        GuestbookEntry {
            id: GuestbookId(id),
            ..Default::default()
        }
    }

    #[test]
    fn append_page_ignores_duplicates_and_deleted_entries() {
        let mut entries = vec![entry(1)];
        append_unique(
            &mut entries,
            vec![entry(1), entry(2), entry(3)],
            Some(GuestbookId(2)),
        );
        assert_eq!(
            entries.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            vec![GuestbookId(1), GuestbookId(3)]
        );
    }
}

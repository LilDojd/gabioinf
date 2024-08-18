use crate::components::{Card, CardType};
use crate::shared::{models::GuestbookEntry, server_fns};
use dioxus::prelude::*;

const SIGNATURES_PER_PAGE: usize = 2;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum SignatureListState {
    #[default]
    Initial,
    Loading,
    Finished,
    MoreAvailable(u32),
}

#[component]
pub fn SignatureList(refresh_trigger: Signal<bool>) -> Element {
    let load_state = use_signal(|| SignatureListState::default());
    let endless_signatures = use_signal(|| vec![]);
    let (load_next_batch, mut refresh) = use_signature_list(load_state, endless_signatures);

    // Watch for changes in the refresh_trigger
    use_effect(move || {
        let _ = refresh_trigger();
        refresh();
    });

    rsx! {
        div {
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                {endless_signatures.read().iter().flatten().map(|entry| rsx! {
                    Card { card_type: CardType::Signature(entry.clone()) }
                })}
            }

            match *load_state.read() {
                SignatureListState::Initial | SignatureListState::Loading => rsx! {
                    div { class: "text-center py-4", "Loading signatures..." }
                },
                SignatureListState::MoreAvailable(_) => rsx! {
                    button {
                        class: "mt-4 px-4 py-2 bg-jet text-stone-100 rounded hover:bg-onyx",
                        onclick: move |_| load_next_batch.send(()),
                        "Load more..."
                    }
                },
                SignatureListState::Finished => rsx! {
                    div { class: "text-center py-4", "No more signatures to load" }
                },
            }
        }
    }
}

fn use_signature_list(
    mut state: Signal<SignatureListState>,
    mut batches: Signal<Vec<Vec<GuestbookEntry>>>,
) -> (Coroutine<()>, impl FnMut()) {
    use futures::StreamExt as _;

    let load_task = use_coroutine(|mut rx: UnboundedReceiver<Option<u32>>| {
        to_owned![state, batches];
        async move {
            while let Some(next_cursor) = rx.next().await {
                let original_state = *state.read();
                state.set(SignatureListState::Loading);

                match server_fns::load_signatures(next_cursor.unwrap_or(1), SIGNATURES_PER_PAGE)
                    .await
                {
                    Ok(signatures) => {
                        if signatures.is_empty() {
                            state.set(SignatureListState::Finished);
                        } else {
                            let next_page = next_cursor.map_or(2, |c| c + 1);
                            state.set(SignatureListState::MoreAvailable(next_page));
                            batches.write().push(signatures);
                        }
                    }
                    Err(error) => {
                        dioxus_logger::tracing::error!("Could not load signatures: {:?}", error);
                        state.set(original_state);
                    }
                }
            }
        }
    });

    let next_task = use_coroutine(|mut rx: UnboundedReceiver<()>| {
        to_owned![state, load_task];
        async move {
            load_task.send(None);
            while rx.next().await.is_some() {
                if let SignatureListState::MoreAvailable(cursor) = *state.read() {
                    load_task.send(Some(cursor));
                }
            }
        }
    });

    let refresh = move || {
        state.set(SignatureListState::Initial);
        batches.write().clear();
        next_task.send(());
    };

    (next_task, refresh)
}

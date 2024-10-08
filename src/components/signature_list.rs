use crate::components::{Card, CardType, CloseButton, Loading};
use crate::shared::{models::GuestbookEntry, server_fns};
use dioxus::prelude::*;
const SIGNATURES_PER_PAGE: usize = 8;
const INTERSECTION_THRESHOLD: f64 = 0.5;
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum SignatureListState {
    #[default]
    Initial,
    Loading(MaybeFirst),
    Finished,
    MoreAvailable(u32),
}
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MaybeFirst {
    #[default]
    First,
    NotFirst,
}

#[component]
pub fn SignatureList() -> Element {
    let load_state = use_signal(SignatureListState::default);
    let mut user_signature = use_context::<Signal<Option<GuestbookEntry>>>();
    let mut endless_signatures = use_signal(std::vec::Vec::new);
    let load_next_batch = use_signature_list(load_state, endless_signatures);
    let mut is_intersecting = use_signal(|| false);
    use_effect(move || {
        let mut eval = eval(
            format!(
                r#"
            const callback = (entries, observer) => {{
                entries.forEach((entry) => {{
                    dioxus.send(entry.isIntersecting);
                }});
            }};

            const options = {{ root: null, threshold: {INTERSECTION_THRESHOLD} }};
            const observer = new IntersectionObserver(callback, options);

            const target = document.getElementById('signature-loader');
            if (target) {{
                observer.observe(target);
            }}

            // Cleanup function
            () => {{
                if (target) {{
                    observer.unobserve(target);
                }}
            }}
            "#,
            )
            .as_ref(),
        );
        spawn(async move {
            while let Ok(is_intersecting_js) = eval.recv().await {
                if let Some(value) = is_intersecting_js.as_bool() {
                    is_intersecting.set(value);
                }
            }
        });
    });
    use_effect(move || {
        if *is_intersecting.read()
            && matches!(*load_state.read(), SignatureListState::MoreAvailable(_))
        {
            load_next_batch.send(());
        }
    });
    rsx! {
        div {
            {
                if user_signature.read().is_some() {
                    rsx! {
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                            Card {
                                card_type: CardType::Signature {
                                    entry: user_signature.read().clone().unwrap(),
                                    close_button: rsx! {
                                        CloseButton {
                                            layout: "absolute top-2 right-2 w-6 h-6",
                                            onclick: move |_| {
                                                spawn(async move {
                                                    server_fns::delete_signature(user_signature.read().clone().unwrap())
                                                        .await
                                                        .unwrap();
                                                    user_signature.set(None);
                                                    endless_signatures.write().clear();
                                                });
                                            },
                                        }
                                    },
                                },
                            }
                            {
                                endless_signatures
                                    .read()
                                    .iter()
                                    .flatten()
                                    .filter(|entry| entry.id != user_signature.read().as_ref().unwrap().id)
                                    .map(|entry| rsx! {
                                        Card {
                                            card_type: CardType::Signature {
                                                entry: entry.clone(),
                                                close_button: rsx! {  },
                                            },
                                        }
                                    })
                            }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                            {
                                endless_signatures
                                    .read()
                                    .iter()
                                    .flatten()
                                    .map(|entry| rsx! {
                                        Card {
                                            card_type: CardType::Signature {
                                                entry: entry.clone(),
                                                close_button: rsx! {  },
                                            },
                                        }
                                    })
                            }
                        }
                    }
                }
            }
            div {
                match *load_state.read() {
                    SignatureListState::Initial | SignatureListState::Loading(MaybeFirst::First) => {
                        rsx! {
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                                {(0..SIGNATURES_PER_PAGE).map(|_| rsx! {
                                    Card { card_type: CardType::Skeleton }
                                })}
                            }
                        }
                    }
                    SignatureListState::Loading(MaybeFirst::NotFirst) => {
                        rsx! {
                            Loading {}
                        }
                    }
                    _ => rsx! {  },
                }
            }
            div { id: "signature-loader", class: "h-5" }
        
        }
    }
}
fn use_signature_list(
    mut state: Signal<SignatureListState>,
    mut batches: Signal<Vec<Vec<GuestbookEntry>>>,
) -> Coroutine<()> {
    use futures::StreamExt as _;
    let load_task = use_coroutine(|mut rx: UnboundedReceiver<Option<u32>>| async move {
        while let Some(next_cursor) = rx.next().await {
            let original_state = *state.read();
            state.set(SignatureListState::Loading(if next_cursor.is_some() {
                MaybeFirst::NotFirst
            } else {
                MaybeFirst::First
            }));
            match server_fns::load_signatures(next_cursor.unwrap_or(1), SIGNATURES_PER_PAGE).await {
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
    });
    use_coroutine(|mut rx: UnboundedReceiver<()>| async move {
        load_task.send(None);
        while rx.next().await.is_some() {
            if let SignatureListState::MoreAvailable(cursor) = *state.read() {
                load_task.send(Some(cursor));
            }
        }
    })
}

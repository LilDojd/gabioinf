use crate::components::{Card, CardType, Loading};
use crate::shared::{models::GuestbookEntry, server_fns};
use dioxus::prelude::*;

const SIGNATURES_PER_PAGE: usize = 8;
const INTERSECTION_THRESHOLD: f64 = 0.8;

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

    // Use the intersection observer
    let loader_ref = use_signal(|| format!("signature-loader-{}", rand::random::<u32>()));

    let is_intersecting = use_signal(|| false);

    use_effect(move || {
        // Create and run the Intersection Observer
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

            const target = document.getElementById('{loader_ref}');
            if (target) {{
                observer.observe(target);
            }}

            // Cleanup function
            () => {{
                if (target) {{
                    observer.unobserve(target);
                }}
            }}
            "#
            )
            .as_ref(),
        );

        to_owned![is_intersecting];
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

    use_effect(move || {
        let _ = refresh_trigger();
        refresh();
    });

    rsx! {
        div {
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                {
                    endless_signatures
                        .read()
                        .iter()
                        .flatten()
                        .map(|entry| rsx! {
                            Card { card_type: CardType::Signature(entry.clone()) }
                        })
                }
            }
            div { id: "{loader_ref}", class: "h-5"}
            div {
                match *load_state.read() {
                    SignatureListState::Initial | SignatureListState::Loading => {
                        rsx! {
                            Loading {}
                        }
                    }
                    SignatureListState::MoreAvailable(_) => rsx! {  },
                    SignatureListState::Finished => rsx! {  },
                }
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

pub fn use_intersection_observer(element_id: String, threshold: f64) -> Signal<bool> {
    let is_intersecting = use_signal(|| false);

    use_effect(move || {
        // Create and run the Intersection Observer
        let mut eval = eval(
            format!(
                r#"
            const callback = (entries, observer) => {{
                entries.forEach((entry) => {{
                    if (entry.isIntersecting == true) {{
                        console.log("Intersecting");
                        console.log(entry);
                        dioxus.send(entry.isIntersecting);
                    }}
                }});
            }};

            const options = {{ root: null, threshold: {threshold} }};
            const observer = new IntersectionObserver(callback, options);

            const target = document.getElementById('{element_id}');
            console.log(target);
            if (target) {{
                observer.observe(target);
            }}

            // Cleanup function
            () => {{
                if (target) {{
                    observer.unobserve(target);
                }}
            }}
            "#
            )
            .as_ref(),
        );

        to_owned![is_intersecting];
        spawn(async move {
            while let Ok(is_intersecting_js) = eval.recv().await {
                if let Some(value) = is_intersecting_js.as_bool() {
                    is_intersecting.set(value);
                }
            }
        });
    });

    is_intersecting
}

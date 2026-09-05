use crate::auth::AuthState;
use crate::components::{Button, ButtonVariant};
use crate::shared::{
    models::{GuestbookCursor, GuestbookEntry, GuestbookId, GuestbookPage},
    server_fns,
};
use dioxus::prelude::*;
use std::rc::Rc;
use time::{OffsetDateTime, macros::format_description};

mod cache;
#[cfg(all(test, feature = "server"))]
mod lifecycle_tests;
pub(crate) use cache::{SignatureCache, spawn_signature_mutation};

const INITIAL_SKELETONS: usize = 6;
const MORE_SKELETONS: usize = 3;
const ENTRY_DATE: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[day padding:none] [month repr:short] [year]");

#[component]
pub fn SignatureList(mut count: Signal<Option<usize>>) -> Element {
    let mut auth_state = use_context::<Signal<Option<AuthState>>>();
    let mut cache = use_context::<Signal<SignatureCache>>();
    let mut entries = use_signal(move || {
        cache.peek().page().map_or_else(Vec::new, |page| {
            page.entries.iter().cloned().map(Rc::new).collect()
        })
    });
    let mut next_cursor = use_signal(move || cache.peek().page().and_then(|page| page.next_cursor));
    let mut loaded_once = use_signal(move || cache.peek().page().is_some());
    let mut refresh_first = use_signal(move || !cache.peek().is_fresh(OffsetDateTime::now_utc()));
    let mut observed_generation = use_signal(move || cache.peek().generation);
    let mut loading = use_signal(|| false);
    let mut load_error = use_signal(|| None::<String>);
    let mut deleting = use_signal(|| false);
    let mut delete_error = use_signal(|| None::<String>);
    let mut deleted_entry_id = use_signal(|| None::<GuestbookId>);

    use_effect(move || {
        let generation = cache.read().generation;
        if generation != *observed_generation.peek() {
            observed_generation.set(generation);
            refresh_first.set(true);
            load_error.set(None);
        }
    });

    // Shared card props avoid repeatedly cloning base64 strings as more pages load.
    let user_entry = use_memo(move || match auth_state.read().as_ref() {
        Some(AuthState::Authenticated(user)) => user.entry.clone().map(Rc::new),
        _ => None,
    });
    let user_entry_id = user_entry.read().as_ref().map(|entry| entry.id);

    use_effect(move || {
        let pinned_id = user_entry.read().as_ref().map(|entry| entry.id);
        count.set(loaded_once().then(|| visible_count(&entries.read(), pinned_id)));
    });

    let load_more = use_callback(move |_| {
        if loading() || (!refresh_first() && loaded_once() && next_cursor().is_none()) {
            return;
        }
        let cursor = if refresh_first() { None } else { next_cursor() };
        let generation = cache.peek().generation;
        loading.set(true);
        load_error.set(None);
        spawn(async move {
            // Always public/unfiltered: authentication must not gate this request or
            // change its cursor. The viewer's pinned entry is deduplicated at render.
            let result = load_public_page(cursor).await;
            if !cache.peek().accepts(generation) {
                refresh_first.set(true);
                loading.set(false);
                return;
            }
            match result {
                Ok(page) => {
                    if cursor.is_none() {
                        cache.write().store(&page, OffsetDateTime::now_utc());
                        entries.write().clear();
                    }
                    append_unique(&mut entries.write(), page.entries, deleted_entry_id());
                    next_cursor.set(page.next_cursor);
                    loaded_once.set(true);
                    refresh_first.set(false);
                }
                Err(error) => {
                    tracing::error!("Could not load signatures: {error:?}");
                    load_error.set(Some(
                        "Could not load signatures. Check your connection and retry.".to_string(),
                    ));
                }
            }
            loading.set(false);
        });
    });

    use_effect(move || {
        if (!loaded_once() || refresh_first()) && !loading() && load_error.read().is_none() {
            load_more.call(());
        }
    });

    rsx! {
        if let Some(error) = delete_error.read().as_ref() {
            div { role: "alert", class: "label-mono text-mars", {error.to_string()} }
        }
        div { class: "grid grid-cols-1 gap-2.5 md:grid-cols-2",
            if let Some(user_entry) = user_entry() {
                {
                    let id = user_entry.id;
                    rsx! {
                        SignatureCard {
                            entry: user_entry,
                            action: Some(rsx! {
                                button {
                                    class: "absolute top-2 right-2 flex size-6 items-center justify-center rounded-full border border-border-strong text-label hover:border-mars hover:text-mars disabled:opacity-50",
                                    aria_label: "Delete your guestbook entry",
                                    disabled: deleting(),
                                    onclick: move |_| {
                                        if deleting() { return; }
                                        deleting.set(true);
                                        delete_error.set(None);
                                        spawn_signature_mutation(cache, server_fns::delete_signature(id), move |result| {
                                            if deleting.try_write().is_err() { return; }
                                            match result {
                                                Ok(()) => {
                                                    deleted_entry_id.set(Some(id));
                                                    entries.write().retain(|entry| entry.id != id);
                                                    if let Some(AuthState::Authenticated(user_state)) = &mut *auth_state.write() {
                                                        user_state.entry = None;
                                                    }
                                                }
                                                Err(error) => {
                                                    tracing::error!("Error deleting signature: {error:?}");
                                                    delete_error.set(Some("Could not delete your signature. Retry with the × button.".to_string()));
                                                }
                                            }
                                            deleting.set(false);
                                        });
                                    },
                                    "×"
                                }
                            })
                        }
                    }
                }
            }
            for entry in entries.read().iter().filter(|entry| Some(entry.id) != user_entry_id) {
                SignatureCard { key: "{entry.id.as_value()}", entry: entry.clone() }
            }
            if !loaded_once() && load_error.read().is_none() {
                for index in 0..INITIAL_SKELETONS {
                    SignatureSkeleton { key: "{index}" }
                }
            }
        }
        if !loaded_once() && load_error.read().is_none() {
            span { role: "status", class: "label-mono", "loading signatures…" }
        }
        if loading() && loaded_once() {
            div {
                role: "status",
                aria_label: if refresh_first() { "Refreshing signatures" } else { "Loading more signatures" },
                class: "grid grid-cols-1 gap-2.5 py-3 sm:grid-cols-3",
                for index in 0..MORE_SKELETONS {
                    SignatureSkeleton { key: "{index}", compact: true }
                }
            }
        } else if let Some(error) = load_error.read().as_ref() {
            div { role: "alert", class: "flex flex-col items-center gap-3 py-5 text-mars",
                span { class: "label-mono text-mars", {error.to_string()} }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| load_more.call(()), "retry" }
            }
        } else if loaded_once() && next_cursor.read().is_some() {
            div {
                aria_hidden: "true",
                class: "h-8",
                onvisible: move |event| {
                    if event.data().is_intersecting().unwrap_or(false) {
                        load_more.call(());
                    }
                },
            }
        }
    }
}

async fn load_public_page(
    cursor: Option<GuestbookCursor>,
) -> Result<GuestbookPage, server_fns::ServerError> {
    #[cfg(all(test, feature = "server"))]
    if let Some(requests) = try_consume_context::<lifecycle_tests::PageRequests>() {
        return requests.load(cursor).await;
    }
    server_fns::load_signatures(cursor, None).await
}

#[derive(Props, Clone, PartialEq)]
struct SignatureCardProps {
    entry: Rc<GuestbookEntry>,
    #[props(default)]
    action: Option<Element>,
}

#[component]
fn SignatureCard(props: SignatureCardProps) -> Element {
    let date = props
        .entry
        .created_at
        .date()
        .format(ENTRY_DATE)
        .expect("the static date format is valid")
        .to_lowercase();
    rsx! {
        article { class: "card relative flex h-full flex-col gap-3.5 p-4",
            if let Some(action) = props.action { {action} }
            p { class: "prose-font m-0 pr-4 text-pretty text-[17px] leading-[1.4] text-text", "{props.entry.message}" }
            if let Some(signature) = props.entry.signature.as_deref().filter(|signature| !signature.is_empty()) {
                div { class: "signature-area flex items-center justify-center overflow-hidden rounded-sm",
                    img { class: "max-h-full max-w-full", src: "data:image/png;base64,{signature}", alt: "Signature by {props.entry.author_username}" }
                }
            }
            span { class: "label-mono mt-auto",
                "by "
                a { href: "https://github.com/{props.entry.author_username}", target: "_blank", rel: "noopener noreferrer", class: "text-muted no-underline hover:text-accent", "{props.entry.author_username}" }
                " · {date}"
            }
        }
    }
}

#[component]
fn SignatureSkeleton(#[props(default)] compact: bool) -> Element {
    rsx! {
        article { class: "card flex animate-pulse flex-col gap-3.5 p-4",
            div { class: "flex flex-col gap-2",
                div { class: "h-3.5 w-5/6 rounded bg-hover-row" }
                div { class: "h-3.5 w-2/3 rounded bg-hover-row" }
            }
            div { class: if compact { "h-10 rounded-sm bg-hover-row" } else { "signature-area rounded-sm bg-hover-row" } }
            div { class: "h-3 w-2/5 rounded bg-hover-row" }
        }
    }
}

fn visible_count(entries: &[Rc<GuestbookEntry>], pinned_id: Option<GuestbookId>) -> usize {
    entries
        .iter()
        .filter(|entry| Some(entry.id) != pinned_id)
        .count()
        + usize::from(pinned_id.is_some())
}

fn append_unique(
    entries: &mut Vec<Rc<GuestbookEntry>>,
    incoming: Vec<GuestbookEntry>,
    hidden: Option<GuestbookId>,
) {
    for entry in incoming {
        if Some(entry.id) != hidden && !entries.iter().any(|current| current.id == entry.id) {
            entries.push(Rc::new(entry));
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
    fn pinned_signature_is_only_counted_once_when_public_loading_finishes_first() {
        let entries = vec![Rc::new(entry(1)), Rc::new(entry(2))];
        assert_eq!(visible_count(&entries, None), 2);
        assert_eq!(visible_count(&entries, Some(GuestbookId(1))), 2);
        assert_eq!(visible_count(&entries, Some(GuestbookId(3))), 3);
    }

    #[cfg(feature = "server")]
    #[test]
    fn cached_signatures_render_while_authentication_is_pending() {
        fn app() -> Element {
            use_context_provider(|| Signal::new(None::<AuthState>));
            use_context_provider(|| {
                let mut cache = SignatureCache::default();
                cache.store(
                    &crate::shared::models::GuestbookPage {
                        entries: vec![GuestbookEntry {
                            message: "Already loaded on the previous visit".into(),
                            ..Default::default()
                        }],
                        next_cursor: None,
                    },
                    OffsetDateTime::now_utc(),
                );
                Signal::new(cache)
            });
            let count = use_signal(|| None);
            rsx! { SignatureList { count } }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);
        assert!(html.contains("Already loaded on the previous visit"));
        assert!(!html.contains("loading signatures…"));
    }

    #[test]
    fn append_page_ignores_duplicates_and_deleted_entries() {
        let mut entries = vec![Rc::new(entry(1))];
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

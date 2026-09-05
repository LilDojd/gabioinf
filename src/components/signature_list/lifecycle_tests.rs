use super::*;
use crate::{
    auth::UserState,
    shared::models::{Guest, GuestId},
};
use dioxus::prelude::dioxus_core::NoOpMutations;
use std::{cell::RefCell, collections::VecDeque};
use tokio::sync::oneshot;

type PageReply = oneshot::Sender<Result<GuestbookPage, server_fns::ServerError>>;
type PendingPages = VecDeque<(Option<GuestbookCursor>, PageReply)>;

#[derive(Clone, Default)]
pub(super) struct PageRequests(Rc<RefCell<PendingPages>>);

impl PageRequests {
    pub(super) async fn load(
        &self,
        cursor: Option<GuestbookCursor>,
    ) -> Result<GuestbookPage, server_fns::ServerError> {
        let (send, receive) = oneshot::channel();
        self.0.borrow_mut().push_back((cursor, send));
        receive.await.expect("test must answer each live request")
    }

    fn next_first_page(&self) -> PageReply {
        let (cursor, reply) = self.0.borrow_mut().pop_front().expect("expected a refresh");
        assert!(cursor.is_none(), "invalidation must refresh the first page");
        reply
    }
}

#[derive(Clone, Copy)]
struct Controls {
    cache: Signal<SignatureCache>,
    auth: Signal<Option<AuthState>>,
    visible: Signal<bool>,
    count: Signal<Option<usize>>,
}

#[derive(Clone, Default)]
struct Harness {
    controls: Rc<RefCell<Option<Controls>>>,
    requests: PageRequests,
}

fn app(harness: Harness) -> Element {
    let cache = use_context_provider(|| {
        let mut cache = SignatureCache::default();
        cache.set_identity(Some(GuestId(1)));
        cache.store(&page(false), OffsetDateTime::now_utc());
        Signal::new(cache)
    });
    let auth = use_context_provider(|| {
        Signal::new(Some(AuthState::Authenticated(Box::new(UserState {
            guest: Guest {
                id: GuestId(1),
                ..Default::default()
            },
            entry: None,
        }))))
    });
    use_context_provider(|| harness.requests.clone());
    let visible = use_signal(|| true);
    let count = use_signal(|| None);
    use_hook(|| {
        *harness.controls.borrow_mut() = Some(Controls {
            cache,
            auth,
            visible,
            count,
        })
    });
    rsx! { if visible() { SignatureList { count } } }
}

fn new_entry() -> GuestbookEntry {
    GuestbookEntry {
        id: GuestbookId(2),
        message: "Newly committed signature".into(),
        ..Default::default()
    }
}

fn page(include_new: bool) -> GuestbookPage {
    let mut entries = vec![GuestbookEntry {
        id: GuestbookId(1),
        message: "Previously visible signature".into(),
        ..Default::default()
    }];
    if include_new {
        entries.insert(0, new_entry());
    }
    GuestbookPage {
        total: entries.len(),
        entries,
        next_cursor: None,
    }
}

// Drain actual component effects and async tasks; a loop in the cache watcher
// fails this bounded check instead of leaving the test running indefinitely.
async fn settle(dom: &mut VirtualDom) {
    for _ in 0..30 {
        dom.render_immediate(&mut NoOpMutations);
        if tokio::time::timeout(std::time::Duration::from_millis(5), dom.wait_for_work())
            .await
            .is_err()
        {
            return;
        }
    }
    panic!("components did not settle");
}

#[tokio::test]
async fn idle_submit_then_sign_out_refreshes_public_cards_without_blanking_them() {
    let harness = Harness::default();
    let mut dom = VirtualDom::new_with_props(app, harness.clone());
    dom.rebuild_in_place();
    settle(&mut dom).await;
    assert!(
        harness.requests.0.borrow().is_empty(),
        "fresh navigation should reuse the page"
    );
    let mut controls = harness.controls.borrow().unwrap();
    assert_eq!(
        controls.count.peek().as_ref(),
        Some(&1),
        "the header counts every signature the guestbook holds, not the loaded rows"
    );
    let (commit, pending) = oneshot::channel();
    dom.in_runtime(|| {
        spawn_signature_mutation(
            controls.cache,
            async move { pending.await.unwrap() },
            move |entry| {
                if let Some(AuthState::Authenticated(user)) = &mut *controls.auth.write() {
                    user.entry = Some(entry);
                }
            },
        )
    });
    settle(&mut dom).await;
    assert!(dioxus::ssr::render(&dom).contains("Previously visible signature"));
    harness
        .requests
        .next_first_page()
        .send(Ok(page(false)))
        .unwrap();
    settle(&mut dom).await;
    assert!(
        controls.cache.peek().page().is_none(),
        "a pre-commit refresh cannot refill the cache"
    );

    commit.send(new_entry()).unwrap();
    settle(&mut dom).await;
    harness
        .requests
        .next_first_page()
        .send(Ok(page(true)))
        .unwrap();
    settle(&mut dom).await;
    dom.in_runtime(|| {
        controls.cache.write().set_identity(None);
        controls.auth.set(Some(AuthState::Unauthenticated));
    });
    settle(&mut dom).await;
    assert!(dioxus::ssr::render(&dom).contains("Newly committed signature"));
    harness
        .requests
        .next_first_page()
        .send(Ok(page(true)))
        .unwrap();
    settle(&mut dom).await;
    assert!(dioxus::ssr::render(&dom).contains("Newly committed signature"));
    assert_eq!(controls.count.peek().as_ref(), Some(&2));
    assert!(
        harness.requests.0.borrow().is_empty(),
        "storing a refreshed page must not start another refresh"
    );
}

#[tokio::test]
async fn navigation_during_mutation_cannot_cache_a_pre_commit_refresh() {
    let harness = Harness::default();
    let mut dom = VirtualDom::new_with_props(app, harness.clone());
    dom.rebuild_in_place();
    settle(&mut dom).await;
    let mut controls = harness.controls.borrow().unwrap();
    let (commit, pending) = oneshot::channel::<()>();
    dom.in_runtime(|| {
        spawn_signature_mutation(
            controls.cache,
            async move { pending.await.unwrap() },
            |_| {},
        )
    });
    settle(&mut dom).await;
    let cancelled_refresh = harness.requests.next_first_page();
    dom.in_runtime(|| controls.visible.set(false));
    settle(&mut dom).await;
    assert!(cancelled_refresh.send(Ok(page(false))).is_err());

    dom.in_runtime(|| controls.visible.set(true));
    settle(&mut dom).await;
    harness
        .requests
        .next_first_page()
        .send(Ok(page(false)))
        .unwrap();
    settle(&mut dom).await;
    assert!(controls.cache.peek().page().is_none());
    dom.in_runtime(|| controls.visible.set(false));
    settle(&mut dom).await;
    commit.send(()).unwrap();
    settle(&mut dom).await;

    dom.in_runtime(|| controls.visible.set(true));
    settle(&mut dom).await;
    harness
        .requests
        .next_first_page()
        .send(Ok(page(true)))
        .unwrap();
    settle(&mut dom).await;
    assert!(dioxus::ssr::render(&dom).contains("Newly committed signature"));
    assert!(
        controls.cache.peek().is_fresh(OffsetDateTime::now_utc()),
        "completion after unmount must re-enable caching"
    );
    assert!(harness.requests.0.borrow().is_empty());
}

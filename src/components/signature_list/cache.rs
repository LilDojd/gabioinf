use crate::shared::models::{GuestId, GuestbookPage};
use dioxus::prelude::{dioxus_core::spawn_forever, *};
use time::{Duration, OffsetDateTime};

const FRESH_FOR: Duration = Duration::seconds(30);
const MAX_ENTRIES: usize = 10;
const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;

/// A single public page, owned by the app's context, never a process-global SSR cache.
/// Only client effects/event handlers populate it; authentication is not cached.
#[derive(Default)]
pub(crate) struct SignatureCache {
    page: Option<(OffsetDateTime, GuestbookPage)>,
    pub(super) generation: u64,
    identity: Option<Option<GuestId>>,
    pending_mutations: usize,
}

/// The server may commit after the initiating page unmounts. Keep cache cleanup
/// alive with the app, and let the caller skip UI updates if its scope was dropped.
pub(crate) fn spawn_signature_mutation<T: 'static>(
    mut cache: Signal<SignatureCache>,
    request: impl Future<Output = T> + 'static,
    complete: impl FnOnce(T) + 'static,
) {
    {
        let mut cache = cache.write();
        cache.pending_mutations += 1;
        cache.invalidate();
    }
    spawn_forever(async move {
        let result = request.await;
        if let Ok(mut cache) = cache.try_write() {
            cache.pending_mutations -= 1;
            cache.invalidate();
        }
        complete(result);
    });
}

impl SignatureCache {
    pub(super) fn page(&self) -> Option<&GuestbookPage> {
        self.page.as_ref().map(|(_, page)| page)
    }

    pub(super) fn is_fresh(&self, now: OffsetDateTime) -> bool {
        self.page.as_ref().is_some_and(|(loaded_at, _)| {
            let age = now - *loaded_at;
            age >= Duration::ZERO && age < FRESH_FOR
        })
    }

    pub(crate) fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.page = None;
    }

    pub(crate) fn set_identity(&mut self, identity: Option<GuestId>) {
        if self.identity.is_some_and(|previous| previous != identity) {
            self.invalidate();
        }
        self.identity = Some(identity);
    }

    /// Reject reads started before a successful mutation, including for the visible list.
    pub(super) fn accepts(&self, generation: u64) -> bool {
        self.generation == generation
    }

    pub(super) fn store(&mut self, page: &GuestbookPage, now: OffsetDateTime) {
        if self.pending_mutations != 0 {
            return;
        }
        let bytes: usize = page
            .entries
            .iter()
            .map(|entry| {
                entry.message.len()
                    + entry.author_username.len()
                    + entry.signature.as_ref().map_or(0, String::len)
            })
            .sum();
        // Keep the entire page or nothing: truncating it would skip rows at its cursor.
        self.page = (page.entries.len() <= MAX_ENTRIES && bytes <= MAX_PAYLOAD_BYTES)
            .then(|| (now, page.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::models::{GuestbookCursor, GuestbookEntry, GuestbookId};

    fn page(id: i64) -> GuestbookPage {
        GuestbookPage {
            entries: vec![GuestbookEntry {
                id: GuestbookId(id),
                message: "Hello!".into(),
                created_at: OffsetDateTime::UNIX_EPOCH,
                updated_at: OffsetDateTime::UNIX_EPOCH,
                ..Default::default()
            }],
            next_cursor: Some(GuestbookCursor {
                id: GuestbookId(id),
                created_at: OffsetDateTime::UNIX_EPOCH,
            }),
        }
    }

    #[test]
    fn return_navigation_reuses_page_and_cursor_but_expiry_requires_refresh() {
        let now = OffsetDateTime::UNIX_EPOCH;
        let mut cache = SignatureCache::default();
        let first = page(1);
        cache.store(&first, now);

        assert!(cache.is_fresh(now + Duration::seconds(29)));
        assert_eq!(cache.page(), Some(&first));
        assert!(!cache.is_fresh(now + Duration::seconds(30)));
        // Stale content is still available while a refresh is pending or fails.
        assert_eq!(cache.page(), Some(&first));
        assert!(!cache.is_fresh(now - Duration::seconds(1)));
    }

    #[test]
    fn successful_mutation_clears_cache_and_rejects_older_in_flight_reads() {
        let mut cache = SignatureCache::default();
        cache.store(&page(1), OffsetDateTime::UNIX_EPOCH);
        let pending_read = cache.generation;

        cache.invalidate();

        assert!(cache.page().is_none());
        assert!(!cache.accepts(pending_read));
        assert!(cache.accepts(cache.generation));
        cache.store(&page(2), OffsetDateTime::UNIX_EPOCH);
        assert_eq!(cache.page().unwrap().entries[0].id, GuestbookId(2));
    }

    #[test]
    fn identity_changes_clear_cached_content_but_same_viewer_can_reuse_it() {
        let mut cache = SignatureCache::default();
        cache.set_identity(Some(GuestId(1)));
        cache.store(&page(1), OffsetDateTime::UNIX_EPOCH);
        cache.set_identity(Some(GuestId(1)));
        assert!(cache.page().is_some());

        let pending_read = cache.generation;
        cache.set_identity(Some(GuestId(2)));
        assert!(cache.page().is_none());
        assert!(!cache.accepts(pending_read));

        cache.store(&page(2), OffsetDateTime::UNIX_EPOCH);
        cache.set_identity(None);
        assert!(cache.page().is_none());
    }

    #[test]
    fn cache_replaces_previous_page_and_skips_oversized_pages_without_truncation() {
        let mut cache = SignatureCache::default();
        cache.store(&page(1), OffsetDateTime::UNIX_EPOCH);
        cache.store(&page(2), OffsetDateTime::UNIX_EPOCH);
        assert_eq!(cache.page(), Some(&page(2)));

        let mut large = page(3);
        large.entries[0].signature = Some("a".repeat(MAX_PAYLOAD_BYTES));
        cache.store(&large, OffsetDateTime::UNIX_EPOCH);
        assert!(cache.page().is_none());

        large.entries = vec![GuestbookEntry::default(); 11];
        cache.store(&large, OffsetDateTime::UNIX_EPOCH);
        assert!(cache.page().is_none());

        large.entries.truncate(10);
        cache.store(&large, OffsetDateTime::UNIX_EPOCH);
        assert_eq!(cache.page(), Some(&large));
    }
}

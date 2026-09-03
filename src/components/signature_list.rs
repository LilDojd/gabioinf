use crate::auth::AuthState;
use crate::components::{Button, ButtonVariant};
use crate::shared::{
    models::{GuestbookEntry, GuestbookId},
    server_fns,
};
use dioxus::prelude::*;
use time::macros::format_description;

const SIGNATURES_PER_PAGE: usize = 10;
const ENTRY_DATE: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[day padding:none] [month repr:short] [year]");

#[component]
pub fn SignatureList(mut count: Signal<usize>) -> Element {
    let mut auth_state = use_context::<dioxus::fullstack::Loader<AuthState>>();
    let mut entries = use_signal(Vec::<GuestbookEntry>::new);
    let mut next_cursor = use_signal(|| None);
    let mut loaded_once = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut load_error = use_signal(|| None::<String>);
    let mut deleting = use_signal(|| false);
    let mut delete_error = use_signal(|| None::<String>);
    let mut deleted_entry_id = use_signal(|| None::<GuestbookId>);

    let user_entry = match &*auth_state.read() {
        AuthState::Authenticated(user_state) => user_state.entry.clone(),
        AuthState::Unauthenticated => None,
    };
    let user_entry_id = user_entry.as_ref().map(|entry| entry.id);

    use_effect(move || {
        let has_user_entry = matches!(
            &*auth_state.read(),
            AuthState::Authenticated(user) if user.entry.is_some()
        );
        count.set(entries.read().len() + usize::from(has_user_entry));
    });

    let load_more = use_callback(move |_| {
        if loading() || (loaded_once() && next_cursor().is_none()) {
            return;
        }
        let cursor = next_cursor();
        loading.set(true);
        load_error.set(None);
        spawn(async move {
            match server_fns::load_signatures(cursor, user_entry_id).await {
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

    rsx! {
        if let Some(error) = delete_error.read().as_ref() {
            div { role: "alert", class: "label-mono text-mars", {error.to_string()} }
        }
        div { class: "grid grid-cols-1 gap-2.5 min-[760px]:grid-cols-2",
            if let Some(user_entry) = user_entry {
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
                                        spawn(async move {
                                            match server_fns::delete_signature(id).await {
                                                Ok(()) => {
                                                    deleted_entry_id.set(Some(id));
                                                    entries.write().retain(|entry| entry.id != id);
                                                    if let AuthState::Authenticated(user_state) = &mut *auth_state.write() {
                                                        user_state.entry = None;
                                                    }
                                                }
                                                Err(error) => {
                                                    dioxus_logger::tracing::error!("Error deleting signature: {error:?}");
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
            if loading() && !loaded_once() {
                for index in 0..SIGNATURES_PER_PAGE - usize::from(user_entry_id.is_some()) {
                    div { key: "{index}", class: "card h-48 animate-pulse p-4",
                        div { class: "h-4 w-3/4 rounded bg-card" }
                    }
                }
            }
        }
        if loading() && loaded_once() {
            span { class: "label-mono block py-5 text-center", "loading…" }
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

#[derive(Props, Clone, PartialEq)]
struct SignatureCardProps {
    entry: GuestbookEntry,
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
                div { class: "flex h-[72px] items-center justify-center overflow-hidden rounded-sm",
                    img { class: "max-h-full max-w-full", src: "data:image/png;base64,{signature}", alt: "Signature by {props.entry.author_username}" }
                }
            } else {
                div { class: "h-[72px] rounded-sm border border-dashed border-[#2c3037] bg-[repeating-linear-gradient(135deg,#1c1f24_0_6px,#191c20_6px_12px)]" }
            }
            span { class: "label-mono mt-auto",
                "by "
                a { href: "https://github.com/{props.entry.author_username}", target: "_blank", rel: "noopener noreferrer", class: "text-muted no-underline hover:text-accent", "{props.entry.author_username}" }
                " · {date}"
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

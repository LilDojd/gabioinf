use crate::{
    auth::AuthState,
    components::{
        Button, ButtonVariant, GithubMark, SignatureCache, SignatureList, SignaturePopup,
        server_error_message, spawn_signature_mutation,
    },
    shared::server_fns,
};
use dioxus::prelude::*;

#[component]
pub fn Guestbook() -> Element {
    let mut auth_state = use_context_provider(|| Signal::new(None::<AuthState>));
    let mut cache = use_context::<Signal<SignatureCache>>();
    let mut auth_error = use_signal(|| None::<String>);
    // Client-only, like the public list's effect: neither request suspends the shell
    // or waits for the other. A pending session must not look signed out.
    let mut auth_request = use_future(move || async move {
        auth_error.set(None);
        match server_fns::load_guestbook_user().await {
            Ok(state) => {
                let identity = match &state {
                    AuthState::Authenticated(user) => Some(user.guest.id),
                    AuthState::Unauthenticated => None,
                };
                cache.write().set_identity(identity);
                auth_state.set(Some(state));
            }
            Err(error) => {
                tracing::error!("Could not load guestbook session: {error:?}");
                auth_error.set(Some("Could not check sign-in status.".to_string()));
            }
        }
    });
    let mut show_popup = use_signal(|| false);
    let mut submitting = use_signal(|| false);
    let mut submit_error = use_signal(|| None::<String>);
    let action_error = use_signal(|| None::<String>);
    let count = use_signal(|| None::<usize>);

    rsx! {
        section { class: "flex flex-col gap-8",
            header { class: "flex flex-col gap-3.5",
                span { class: "label-mono",
                    match count() {
                        Some(1) => rsx! { "// guestbook · 1 signature" },
                        Some(count) => rsx! { "// guestbook · {count} signatures" },
                        None => rsx! { "// guestbook" },
                    }
                }
                h1 { class: "heading-casual m-0 text-[30px] leading-[1.2]", "sign my guestbook" }
                div { class: "flex flex-wrap gap-2",
                    match auth_state.read().as_ref() {
                        Some(AuthState::Authenticated(user)) if user.entry.is_none() => rsx! {
                            Button { onclick: move |_| { submit_error.set(None); show_popup.set(true); }, "sign guestbook" }
                            SignOutButton { auth_state, action_error }
                        },
                        Some(AuthState::Authenticated(_)) => rsx! { SignOutButton { auth_state, action_error } },
                        Some(AuthState::Unauthenticated) => rsx! {
                            a { href: "/v1/login?next=/guestbook", class: "btn-primary",
                                GithubMark { size: 16 }
                                "sign in with github"
                            }
                        },
                        None => rsx! {
                            if let Some(error) = auth_error.read().as_ref() {
                                span { role: "alert", class: "label-mono text-mars", {error.to_string()} }
                                Button { variant: ButtonVariant::Secondary, onclick: move |_| auth_request.restart(), "retry sign-in status" }
                            } else {
                                span { role: "status", class: "label-mono", "checking sign-in…" }
                            }
                        },
                    }
                }
                if let Some(error) = action_error.read().as_ref() {
                    span { role: "alert", class: "label-mono text-mars", {error.to_string()} }
                }
            }
            if show_popup() {
                SignaturePopup {
                    submitting: submitting(),
                    submit_error: submit_error(),
                    on_close: move |_| {
                        if !submitting() { show_popup.set(false); }
                    },
                    on_submit: move |request: server_fns::CreateEntryRequest| {
                        if submitting() || !matches!(&*auth_state.read(), Some(AuthState::Authenticated(_))) {
                            return;
                        }
                        submitting.set(true);
                        submit_error.set(None);
                        spawn_signature_mutation(cache, server_fns::submit_signature(request), move |result| {
                            if submitting.try_write().is_err() { return; }
                            match result {
                                Ok(entry) => {
                                    if let Some(AuthState::Authenticated(user)) = &mut *auth_state.write() {
                                        user.entry = Some(entry);
                                    }
                                    show_popup.set(false);
                                }
                                Err(error) => {
                                    tracing::error!("Error submitting signature: {error:?}");
                                    submit_error.set(Some(server_error_message(&error, "Could not sign the guestbook")));
                                }
                            }
                            submitting.set(false);
                        });
                    },
                }
            }
            SignatureList { count }
        }
    }
}

#[component]
fn SignOutButton(
    mut auth_state: Signal<Option<AuthState>>,
    mut action_error: Signal<Option<String>>,
) -> Element {
    let mut cache = use_context::<Signal<SignatureCache>>();
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            onclick: move |_| {
                action_error.set(None);
                spawn(async move {
                    match server_fns::logout().await {
                        Ok(()) => {
                            cache.write().set_identity(None);
                            auth_state.set(Some(AuthState::Unauthenticated));
                        },
                        Err(error) => {
                            tracing::error!("Could not sign out: {error:?}");
                            action_error.set(Some("Could not sign out. Please retry.".to_string()));
                        }
                    }
                });
            },
            "sign out"
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn shell_and_list_placeholders_render_without_a_session_or_database() {
        fn app() -> Element {
            use_context_provider(|| Signal::new(SignatureCache::default()));
            rsx! { Guestbook {} }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);

        assert!(html.contains("sign my guestbook"));
        assert!(html.contains("loading signatures…"));
        assert!(html.contains("checking sign-in…"));
        assert!(!html.contains("sign in with github"));
        assert!(!html.contains("0 signatures"));
    }
}

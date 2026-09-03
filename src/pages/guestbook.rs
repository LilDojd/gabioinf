use crate::{
    auth::AuthState,
    components::{
        Button, ButtonVariant, GithubMark, SignatureList, SignaturePopup, server_error_message,
    },
    shared::server_fns,
};
use dioxus::prelude::*;

#[component]
pub fn Guestbook() -> Element {
    let auth_state = use_loader(|| async {
        let Some(user) = server_fns::get_user().await? else {
            return Ok::<_, server_fns::ServerError>(AuthState::Unauthenticated);
        };
        let entry = server_fns::load_user_signature(user.clone()).await?;
        Ok(AuthState::Authenticated(Box::new(crate::auth::UserState {
            guest: user,
            entry,
        })))
    })?;
    use_context_provider(|| auth_state);

    rsx! { GuestbookContent {} }
}

#[component]
fn GuestbookContent() -> Element {
    let mut auth_state = use_context::<dioxus::fullstack::Loader<AuthState>>();
    let mut show_popup = use_signal(|| false);
    let mut submitting = use_signal(|| false);
    let mut submit_error = use_signal(|| None::<String>);
    let action_error = use_signal(|| None::<String>);
    let count = use_signal(|| 0usize);

    rsx! {
        section { class: "flex flex-col gap-8",
            header { class: "flex flex-col gap-3.5",
                span { class: "label-mono", "// guestbook · {count} signatures" }
                h1 { class: "heading-casual m-0 text-[30px] leading-[1.2]", "sign my guestbook" }
                p { class: "prose-font m-0 text-pretty text-lg text-muted",
                    "Leave a note and a doodle. Signing in with GitHub keeps the bots out; nothing else is stored."
                }
                div { class: "flex flex-wrap gap-2",
                    match &*auth_state.read() {
                        AuthState::Authenticated(user) if user.entry.is_none() => rsx! {
                            Button { onclick: move |_| { submit_error.set(None); show_popup.set(true); }, "sign guestbook" }
                            SignOutButton { auth_state, action_error }
                        },
                        AuthState::Authenticated(_) => rsx! { SignOutButton { auth_state, action_error } },
                        AuthState::Unauthenticated => rsx! {
                            a { href: "/v1/login?next=/guestbook", class: "btn-primary",
                                GithubMark { size: 16 }
                                "sign in with github"
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
                    on_submit: move |request: server_fns::CreateEntryRequest| async move {
                        if submitting() || !matches!(&*auth_state.read(), AuthState::Authenticated(_)) {
                            return;
                        }
                        submitting.set(true);
                        submit_error.set(None);
                        match server_fns::submit_signature(request).await {
                            Ok(entry) => {
                                if let AuthState::Authenticated(user) = &mut *auth_state.write() {
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
                    },
                }
            }
            SignatureList { count }
        }
    }
}

#[component]
fn SignOutButton(
    mut auth_state: dioxus::fullstack::Loader<AuthState>,
    mut action_error: Signal<Option<String>>,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            onclick: move |_| {
                action_error.set(None);
                spawn(async move {
                    match server_fns::logout().await {
                        Ok(()) => auth_state.set(AuthState::Unauthenticated),
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

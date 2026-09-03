use crate::{
    auth::AuthState,
    components::{Button, ButtonVariant, SignatureList, SignaturePopup},
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
                                GithubMark {}
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
                                dioxus_logger::tracing::error!("Error submitting signature: {error:?}");
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
                            dioxus_logger::tracing::error!("Could not sign out: {error:?}");
                            action_error.set(Some("Could not sign out. Please retry.".to_string()));
                        }
                    }
                });
            },
            "sign out"
        }
    }
}

#[component]
fn GithubMark() -> Element {
    rsx! {
        svg { width: "16", height: "16", view_box: "0 0 98 98", "aria-hidden": "true",
            path { fill: "currentColor", fill_rule: "evenodd", clip_rule: "evenodd", d: "M48.854 0C21.839 0 0 22 0 49.217c0 21.756 13.993 40.172 33.405 46.69 2.427.49 3.316-1.059 3.316-2.362 0-1.141-.08-5.052-.08-9.127-13.59 2.934-16.42-5.867-16.42-5.867-2.184-5.704-5.42-7.17-5.42-7.17-4.448-3.015.324-3.015.324-3.015 4.934.326 7.523 5.052 7.523 5.052 4.367 7.496 11.404 5.378 14.235 4.074.404-3.178 1.699-5.378 3.074-6.6-10.839-1.141-22.243-5.378-22.243-24.283 0-5.378 1.94-9.778 5.014-13.2-.485-1.222-2.184-6.275.486-13.038 0 0 4.125-1.304 13.426 5.052a46.97 46.97 0 0 1 12.214-1.63c4.125 0 8.33.571 12.213 1.63 9.302-6.356 13.427-5.052 13.427-5.052 2.67 6.763.97 11.816.485 13.038 3.155 3.422 5.015 7.822 5.015 13.2 0 18.905-11.404 23.06-22.324 24.283 1.78 1.548 3.316 4.481 3.316 9.126 0 6.6-.08 11.897-.08 13.526 0 1.304.89 2.853 3.316 2.364 19.412-6.52 33.405-24.935 33.405-46.691C97.707 22 75.788 0 48.854 0z" }
        }
    }
}

fn server_error_message(error: &server_fns::ServerError, fallback: &str) -> String {
    match error {
        server_fns::ServerError::Internal | server_fns::ServerError::Unavailable => {
            fallback.to_string()
        }
        error => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_validation_but_hides_internal_errors() {
        assert_eq!(
            server_error_message(
                &server_fns::ServerError::Validation("Message is required".to_string()),
                "fallback"
            ),
            "Message is required"
        );
        assert_eq!(
            server_error_message(&server_fns::ServerError::Internal, "Try again"),
            "Try again"
        );
    }
}

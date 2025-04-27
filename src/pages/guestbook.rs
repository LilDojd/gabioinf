use crate::{
    MessageValid,
    auth::AuthState,
    components::{ButtonVariant, IconVariant, SignatureList, SignaturePopup, StyledButton},
    shared::server_fns,
};
use dioxus::prelude::*;
#[component]
pub fn Guestbook() -> Element {
    let mut message_valid = use_context::<Signal<MessageValid>>();

    let mut auth_state = use_context::<Signal<AuthState>>();
    let mut show_signature_pad = use_signal(|| false);
    let close_popup = move |_| show_signature_pad.set(false);

    rsx! {
        div { class: "container mx-auto px-4 py-8",
            article { class: "prose prose-invert prose-stone prose-h2:mb-0 lg:prose-lg mb-8",
                h1 { class: "text-3xl font-bold mb-6", "sign my guestbook" }
            }
            div { class: "mb-6 flex w-full justify-between items-center",
                {
                    match &*auth_state.read() {
                        AuthState::Authenticated(user_state) => {
                            match &user_state.entry {
                                None => rsx! {
                                    StyledButton {
                                        text: "Sign Guestbook",
                                        variant: ButtonVariant::Primary,
                                        onclick: move |_| show_signature_pad.set(true),
                                    }
                                    StyledButton {
                                        text: "Sign out",
                                        variant: ButtonVariant::Secondary,
                                        onclick: move |_| {
                                            spawn(async move {
                                                server_fns::logout().await.unwrap();
                                                auth_state.set(AuthState::Unauthenticated);
                                            });
                                        },
                                        icon: IconVariant::Rsx(rsx! {
                                            svg {
                                                fill: "none",
                                                height: "20",
                                                view_box: "0 0 24 24",
                                                width: "20",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                path {
                                                    stroke: "#f5f5f4",
                                                    stroke_width: "2",
                                                    d: "M17 16L21 12M21 12L17 8M21 12L7 12M13 16V17C13 18.6569 11.6569 20 10 20H6C4.34315 20 3 18.6569 3 17V7C3 5.34315 4.34315 4 6 4H10C11.6569 4 13 5.34315 13 7V8",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                }
                                            }
                                        }),
                                    }
                                },
                                Some(_signature) => rsx! {
                                    StyledButton {
                                        text: "Sign out",
                                        variant: ButtonVariant::Secondary,
                                        onclick: move |_| {
                                            spawn(async move {
                                                server_fns::logout().await.unwrap();
                                                auth_state.set(AuthState::Unauthenticated);
                                            });
                                        },
                                        icon: IconVariant::Rsx(rsx! {
                                            svg {
                                                fill: "none",
                                                height: "20",
                                                view_box: "0 0 24 24",
                                                width: "20",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                path {
                                                    stroke: "#f5f5f4",
                                                    stroke_width: "2",
                                                    d: "M17 16L21 12M21 12L17 8M21 12L7 12M13 16V17C13 18.6569 11.6569 20 10 20H6C4.34315 20 3 18.6569 3 17V7C3 5.34315 4.34315 4 6 4H10C11.6569 4 13 5.34315 13 7V8",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                }
                                            }
                                        }),
                                    }
                                },
                            }
                        }
                        _ => rsx! {
                            a { href: "/v1/login?next=/guestbook",
                                StyledButton {
                                    text: "Sign in with GitHub",
                                    variant: ButtonVariant::Primary,
                                    onclick: |_| (),
                                    icon: IconVariant::Rsx(rsx! {
                                        svg {
                                            xmlns: "http://www.w3.org/2000/svg",
                                            width: "20",
                                            height: "20",
                                            view_box: "0 0 98 98",
                                            path {
                                                d: "M48.854 0C21.839 0 0 22 0 49.217c0 21.756 13.993 40.172 33.405 46.69 2.427.49 3.316-1.059 3.316-2.362 0-1.141-.08-5.052-.08-9.127-13.59 2.934-16.42-5.867-16.42-5.867-2.184-5.704-5.42-7.17-5.42-7.17-4.448-3.015.324-3.015.324-3.015 4.934.326 7.523 5.052 7.523 5.052 4.367 7.496 11.404 5.378 14.235 4.074.404-3.178 1.699-5.378 3.074-6.6-10.839-1.141-22.243-5.378-22.243-24.283 0-5.378 1.94-9.778 5.014-13.2-.485-1.222-2.184-6.275.486-13.038 0 0 4.125-1.304 13.426 5.052a46.97 46.97 0 0 1 12.214-1.63c4.125 0 8.33.571 12.213 1.63 9.302-6.356 13.427-5.052 13.427-5.052 2.67 6.763.97 11.816.485 13.038 3.155 3.422 5.015 7.822 5.015 13.2 0 18.905-11.404 23.06-22.324 24.283 1.78 1.548 3.316 4.481 3.316 9.126 0 6.6-.08 11.897-.08 13.526 0 1.304.89 2.853 3.316 2.364 19.412-6.52 33.405-24.935 33.405-46.691C97.707 22 75.788 0 48.854 0z",
                                                clip_rule: "evenodd",
                                                fill_rule: "evenodd",
                                                fill: "#fff",
                                            }
                                        }
                                    }),
                                }
                            }
                        },
                    }
                }
            }
            {
                if *show_signature_pad.read() {
                    rsx! {
                        SignaturePopup {
                            on_close: close_popup,
                            on_submit: move |(message, signature): (String, String)| async move {
                                let state_: &mut AuthState = &mut auth_state.write();
                                if let AuthState::Authenticated(user_state) = state_ {
                                    let entry_request = server_fns::CreateEntryRequest {
                                        message,
                                        signature: if signature.is_empty() { None } else { Some(signature) },
                                    };
                                    dioxus_logger::tracing::debug!("Submitting signature");
                                    let resp = server_fns::submit_signature(
                                            entry_request,
                                            user_state.guest.clone(),
                                        )
                                        .await;
                                    match resp {
                                        Ok(Some(entry)) => {
                                            message_valid.write().0 = true;
                                            message_valid.write().1 = String::new();
                                            show_signature_pad.set(false);
                                            user_state.entry = Some(entry);
                                        }
                                        Err(e) => {
                                            message_valid.write().0 = false;
                                            if let Some(error) = e
                                                .to_string()
                                                .strip_prefix("error running server function: message: ")
                                            {
                                                message_valid.write().1 = error.to_string();
                                            } else {
                                                message_valid.write().1 = "An internal error occurred"
                                                    .to_string();
                                            }
                                            dioxus_logger::tracing::error!(
                                                "Error submitting signature: {:?}", e
                                            );
                                        }
                                        Ok(None) => {
                                            show_signature_pad.set(false);
                                        }
                                    }
                                } else {
                                    show_signature_pad.set(false);
                                }
                            },
                        }
                    }
                } else {
                    rsx! {}
                }
            }
            SignatureList {}
        }
    }
}

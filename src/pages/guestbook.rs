use crate::{
    components::{ButtonVariant, Card, CardType, SignatureList, SignaturePopup, StyledButton},
    shared::{
        models::{Guest, GuestbookEntry},
        server_fns,
    },
    MessageValid,
};

use dioxus::prelude::*;
const GITHUB_ICON: &str = asset!("assets/github-mark-white.svg");
const LOGOUT: &str = asset!("assets/logout.svg");
#[component]
pub fn Guestbook() -> Element {
    let mut message_valid = use_context::<Signal<MessageValid>>();
    let mut user_signature = use_context::<Signal<Option<GuestbookEntry>>>();

    let mut show_signature_pad = use_signal(|| false);
    let close_popup = move |_| show_signature_pad.set(false);

    let mut user = use_resource(server_fns::get_user);

    rsx! {
        div { class: "container mx-auto px-4 py-8",
            article { class: "prose prose-invert prose-stone prose-h2:mb-0 lg:prose-lg mb-8",
                h1 { class: "text-3xl font-bold mb-6", "sign my guestbook" }
            }
            div { class: "mb-6 flex w-full justify-between items-center",
                {
                    match &*user.read() {
                        Some(Ok(Some(_user))) => rsx! {
                            StyledButton {
                                text: "Sign Guestbook",
                                variant: ButtonVariant::Primary,
                                onclick: move |_| show_signature_pad.set(true),
                            }
                            StyledButton {
                                text: "Sign out",
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| async move {
                                    server_fns::logout().await.unwrap();
                                    user.restart();
                                },
                                icon: Some(LOGOUT.to_string()),
                            }
                        },
                        _ => rsx! {
                            a { href: "/v1/login",
                                StyledButton {
                                    text: "Sign in with GitHub",
                                    variant: ButtonVariant::Primary,
                                    onclick: |_| (),
                                    icon: Some(GITHUB_ICON.to_string()),
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
                                if let Some(Ok(Some(guest))) = &*user.read() {
                                    let entry_request = server_fns::CreateEntryRequest {
                                        message,
                                        signature: if signature.is_empty() { None } else { Some(signature) },
                                    };
                                    dioxus_logger::tracing::info!("Submitting signature");
                                    let resp = server_fns::submit_signature(entry_request, guest.clone()).await;
                                    match resp {
                                        Ok(Some(entry)) => {
                                            message_valid.write().0 = true;
                                            message_valid.write().1 = String::new();
                                            show_signature_pad.set(false);
                                            user_signature.set(Some(entry.clone()));
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
                    rsx! {  }
                }
            }
            SignatureList {}
        }
    }
}

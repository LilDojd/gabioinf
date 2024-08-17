use crate::{
    components::{ButtonVariant, Card, CardType, SignaturePopup, StyledButton},
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
    let mut user = use_server_future(server_fns::get_user)?;
    let mut message_valid = use_context::<Signal<MessageValid>>();

    let mut show_signature_pad = use_signal(|| false);
    let mut messages = use_signal(Vec::<GuestbookEntry>::new);
    let close_popup = move |_| show_signature_pad.set(false);
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
                                    let resp = server_fns::submit_signature(entry_request, guest.clone()).await;
                                    match resp {
                                        Ok(Some(_entry)) => {
                                            message_valid.write().0 = true;
                                            show_signature_pad.set(false);
                                        }
                                        Ok(None) => {
                                            message_valid.write().0 = false;
                                        }
                                        _ => {
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
            {
                if messages.is_empty() {
                    rsx! {
                        div { class: "text-stone-400 text-center",
                            p { "No messages yet. Be the first to sign the guestbook!" }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                            {messages.iter().map(|entry| rsx! {
                                Card { card_type: CardType::Signature(entry.clone()) }
                            })}
                        }
                    }
                }
            }
        }
    }
}

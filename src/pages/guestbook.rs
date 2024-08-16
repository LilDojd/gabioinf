use crate::components::{
    ButtonVariant, Card, CardType, GuestbookEntry, SignaturePopup, StyledButton,
};
use dioxus::prelude::*;
const GITHUB_ICON: &str = asset!("assets/github-mark-white.svg");
const LOGOUT: &str = asset!("assets/logout.svg");
#[component]
pub fn Guestbook() -> Element {
    let mut signed_in = use_signal(|| false);
    let mut show_signature_pad = use_signal(|| false);
    let mut messages = use_signal(Vec::<GuestbookEntry>::new);
    let placeholder_messages = [GuestbookEntry {
        message: "Great website!".to_string(),
        signature: "John Doe".to_string(),
        date: "2024-08-15".to_string(),
        username: "johndoe".to_string(),
    }];
    let mut user_signature = use_signal(|| None::<GuestbookEntry>);
    let close_popup = move |_| show_signature_pad.set(false);
    let submit_signature = move |(message, signature): (String, String)| {
        user_signature.set(Some(GuestbookEntry {
            message,
            signature,
            date: chrono::Local::now().format("%b %d, %Y %I %p").to_string(),
            username: "Current User".to_string(),
        }));
        messages.push(user_signature.read().as_ref().unwrap().clone());
        show_signature_pad.set(false);
    };
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            article { class: "prose prose-invert prose-stone prose-h2:mb-0 lg:prose-lg mb-8",
                h1 { class: "text-3xl font-bold mb-6", "sign my guestbook" }
            }
            div { class: "mb-6 flex w-full justify-between items-center",
                {
                    if !*signed_in.read() {
                        rsx! {
                            StyledButton {
                                text: "Sign in with GitHub",
                                variant: ButtonVariant::Primary,
                                onclick: move |_| signed_in.set(true),
                                icon: Some(GITHUB_ICON.to_string()),
                            }
                        }
                    } else {
                        rsx! {
                            StyledButton {
                                text: "Sign Guestbook",
                                variant: ButtonVariant::Primary,
                                onclick: move |_| show_signature_pad.set(true),
                            }
                            StyledButton {
                                text: "Sign out",
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| {
                                    signed_in.set(false);
                                    show_signature_pad.set(false);
                                },
                                icon: Some(LOGOUT.to_string()),
                            }
                        }
                    }
                }
            }
            {
                if *show_signature_pad.read() {
                    rsx! {
                        SignaturePopup { on_close: close_popup, on_submit: submit_signature }
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

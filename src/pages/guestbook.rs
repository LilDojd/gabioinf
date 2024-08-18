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

    let mut show_signature_pad = use_signal(|| false);
    let close_popup = move |_| show_signature_pad.set(false);
    let mut should_refresh = use_signal(|| false);

    // Since we bubble up the suspense with `?`, the server will wait for the future to resolve before rendering
    let mut user = use_server_future(server_fns::get_user)?;

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
                                            should_refresh.toggle();
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
            SignatureList { refresh_trigger: should_refresh }
        }
    }
}

fn is_near_bottom() -> bool {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let scroll_top = document.document_element().unwrap().scroll_top();
    let scroll_height = document.document_element().unwrap().scroll_height();
    let client_height = document.document_element().unwrap().client_height();

    scroll_top + client_height >= scroll_height - 200
}

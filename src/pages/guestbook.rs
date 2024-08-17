use crate::{
    components::{ButtonVariant, Card, CardType, GuestbookEntry, SignaturePopup, StyledButton},
    shared::models::Guest,
};

#[cfg(feature = "server")]
use crate::backend::domain::logic::SessionWrapper;

use dioxus::prelude::*;
const GITHUB_ICON: &str = asset!("assets/github-mark-white.svg");
const LOGOUT: &str = asset!("assets/logout.svg");
#[component]
pub fn Guestbook() -> Element {
    let mut user = use_server_future(get_user)?;

    let mut show_signature_pad = use_signal(|| false);
    let mut messages = use_signal(Vec::<GuestbookEntry>::new);
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
                                    if let Ok(Some(_user)) = get_user().await {
                                        logout().await.unwrap();
                                        user.restart();
                                    }
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

#[server(GetUserName)]
pub async fn get_user() -> Result<Option<Guest>, ServerFnError> {
    let session: SessionWrapper = extract().await?;

    match session.session.user {
        Some(user) => Ok(Some(user)),
        None => Ok(None),
    }
}

#[server(Logout)]
pub async fn logout() -> Result<(), ServerFnError> {
    let mut session: SessionWrapper = extract().await?;
    dioxus_logger::tracing::info!("Logging out");

    session.session.logout().await?;
    Ok(())
}

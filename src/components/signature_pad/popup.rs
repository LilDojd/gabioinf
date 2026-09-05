use crate::{
    components::{Button, ButtonVariant, SignaturePad},
    shared::server_fns::CreateEntryRequest,
};
use dioxus::prelude::*;

const MAX_MESSAGE_LENGTH: usize = 255;

#[derive(Props, Clone, PartialEq)]
pub struct SignaturePopupProps {
    pub on_close: EventHandler<()>,
    pub on_submit: EventHandler<CreateEntryRequest>,
    pub submitting: bool,
    #[props(default)]
    pub submit_error: Option<String>,
}

#[component]
pub fn SignaturePopup(props: SignaturePopupProps) -> Element {
    let mut message = use_signal(String::new);
    let mut validation = use_signal(|| None::<String>);
    let mut signature = use_signal(|| None::<String>);
    let character_count = message().chars().count();

    let submit = move |_| {
        if props.submitting {
            return;
        }
        let trimmed = message().trim().to_string();
        if trimmed.is_empty() {
            validation.set(Some("Message is required".to_string()));
            return;
        }
        props.on_submit.call(CreateEntryRequest {
            message: trimmed,
            signature: signature(),
        });
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-[rgba(10,11,13,.7)] p-5 backdrop-blur-sm",
            onclick: move |_| {
                if !props.submitting { props.on_close.call(()); }
            },
            div {
                class: "w-full max-w-lg rounded-md border border-border-strong bg-surface p-5 shadow-[0_20px_60px_rgba(0,0,0,.5)]",
                onclick: move |event| event.stop_propagation(),
                h2 { class: "heading-casual m-0 mb-4 text-xl", "sign guestbook" }
                form { class: "flex flex-col gap-4", onsubmit: move |event| event.prevent_default(),
                    label { class: "label-mono flex flex-col gap-2",
                        "leave a message"
                        div { class: "relative",
                            textarea {
                                class: if validation.read().is_some() { "prose-font min-h-24 w-full resize-y rounded-md border border-mars bg-code p-3 pb-7 text-base text-text outline-none placeholder:text-faint focus:border-mars" } else { "prose-font min-h-24 w-full resize-y rounded-md border border-card bg-code p-3 pb-7 text-base text-text outline-none placeholder:text-faint focus:border-accent" },
                                placeholder: "wow, you are the coolest dude i have ever seen...",
                                maxlength: MAX_MESSAGE_LENGTH,
                                disabled: props.submitting,
                                value: message,
                                oninput: move |event| {
                                    let value = event.value();
                                    if value.chars().count() <= MAX_MESSAGE_LENGTH {
                                        message.set(value);
                                        validation.set(None);
                                    }
                                },
                            }
                            span { class: "absolute right-2 bottom-1 text-[11px] text-label", "{character_count} / {MAX_MESSAGE_LENGTH}" }
                        }
                    }
                    if let Some(error) = validation.read().as_ref().or(props.submit_error.as_ref()) {
                        span { role: "alert", class: "label-mono text-mars", {error.to_string()} }
                    }
                    label { class: "label-mono flex flex-col gap-2",
                        "sign here (optional)"
                        SignaturePad {
                            class: "h-48 w-full rounded-md border border-card bg-code",
                            disabled: props.submitting,
                            on_change: move |png| signature.set(png),
                        }
                    }
                    div { class: "flex justify-end gap-2",
                        Button { variant: ButtonVariant::Secondary, disabled: props.submitting, onclick: move |_| props.on_close.call(()), "cancel" }
                        Button { disabled: props.submitting, onclick: submit, if props.submitting { "signing…" } else { "sign" } }
                    }
                }
            }
        }
    }
}

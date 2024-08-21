use crate::{
    components::{signature_pad::Canvas, ButtonVariant, SignaturePad, StyledButton},
    MessageValid,
};
use dioxus::prelude::*;
const MAX_MESSAGE_LENGTH: usize = 255;
#[derive(Props, Debug, Clone, PartialEq)]
pub struct SignaturePopupProps {
    on_close: EventHandler<()>,
    on_submit: EventHandler<(String, String)>,
}
#[component]
pub fn SignaturePopup(props: SignaturePopupProps) -> Element {
    let mut message = use_signal(String::new);
    let mut char_count = use_signal(|| 0);
    let mut local_signature = use_signal(String::new);
    let mut canvas_ref = use_signal(|| None::<Canvas>);
    let trim_on_submit = use_callback(move |_| {
        if let Some(canvas) = canvas_ref.read().as_ref() {
            let trimmed_signature = canvas.trim_to_image();
            props.on_submit.call((message.read().clone(), trimmed_signature));
        }
    });
    let mut message_valid = use_context::<Signal<MessageValid>>();
    let update_message = move |evt: Event<FormData>| {
        let new_message = evt.value();
        match new_message.chars().count() {
            n if n < MAX_MESSAGE_LENGTH => {
                message.set(new_message.clone());
                char_count.set(n);
                message_valid.write().0 = true;
            }
            MAX_MESSAGE_LENGTH => {
                message.set(new_message.clone());
                char_count.set(MAX_MESSAGE_LENGTH);
                message_valid.write().0 = false;
                message_valid.write().1 = "Too long".to_string();
            }
            _ => {
                message_valid.write().0 = false;
                message_valid.write().1 = "Too long".to_string();
            }
        }
    };
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-nasty-black rounded-lg p-6 sm:max-w-lg w-full min-w-0 border border-onyx shadow-lg",
                h2 { class: "text-xl font-bold mb-4 text-stone-100", "Sign guestbook" }
                form {
                    class: "space-y-4",
                    onsubmit: move |evt| evt.prevent_default(),
                    div {
                        label { class: "block text-stone-400 mb-2", "leave a message" }
                        div { class: "relative",
                            textarea {
                                class: if message_valid.read().0 { "border-onyx focus:border-alien-green" } else { "border-coral focus:border-coral" },
                                class: "w-full p-2 pb-6 placeholder:italic placeholder:text-[#434343] rounded-md bg-jet text-stone-100 border focus:outline-none",
                                placeholder: "wow, you are the coolest dude i have ever seen...",
                                rows: "3",
                                maxlength: MAX_MESSAGE_LENGTH.to_string(),
                                oninput: update_message,
                            }
                            span {
                                class: "absolute bottom-2 right-2 text-xs",
                                class: if *char_count.read() == MAX_MESSAGE_LENGTH { "text-coral" } else { "text-stone-400" },
                                "{char_count} / {MAX_MESSAGE_LENGTH}"
                            }
                            if !message_valid.read().0 {
                                span { class: "absolute bottom-2 left-2 text-coral text-xs",
                                    "{message_valid.read().1}"
                                }
                            }
                        }
                    }
                    div {
                        label { class: "block text-stone-400 mb-2", "sign here" }
                        SignaturePad {
                            class: "border bg-jet border-onyx w-full h-48 rounded-md",
                            container_class: "w-full",
                            disabled: false,
                            on_change: move |value: Option<String>| {
                                local_signature.set(value.unwrap_or_default());
                            },
                            on_canvas_ready: move |canvas: Canvas| {
                                canvas_ref.set(Some(canvas));
                            },
                        }
                    }
                    div { class: "flex justify-end space-x-4",
                        StyledButton {
                            text: "Cancel",
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| props.on_close.call(()),
                        }
                        StyledButton {
                            text: "Sign",
                            variant: ButtonVariant::Primary,
                            onclick: trim_on_submit,
                        }
                    }
                }
            }
        }
    }
}

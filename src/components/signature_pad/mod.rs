//! A small freehand pad for guestbook doodles.

mod canvas;
mod point;
mod popup;
mod stroke;
mod utils;

use canvas::{Canvas, Ink};
use dioxus::prelude::*;
pub use popup::SignaturePopup;

#[derive(Props, PartialEq, Clone)]
pub struct SignaturePadProps {
    #[props(default)]
    class: String,
    #[props(default)]
    disabled: bool,
    /// Fires after every completed stroke, undo or clear with the trimmed PNG
    /// (base64) of the whole drawing, or `None` when the pad is empty.
    on_change: EventHandler<Option<String>>,
}

#[component]
pub fn SignaturePad(props: SignaturePadProps) -> Element {
    let mut canvas = use_signal(|| None::<Canvas>);
    let mut ink = use_signal(Ink::default);

    let emit_change = move || {
        let png = canvas.read().as_ref().and_then(Canvas::trimmed_png);
        props.on_change.call(png);
    };
    // Every pointer/undo/clear handler needs the same "borrow the canvas mutably" dance.
    let mut with_canvas = move |edit: fn(&mut Canvas, &PointerEvent), event: PointerEvent| {
        if let Some(canvas) = canvas.write().as_mut() {
            edit(canvas, &event);
        }
    };

    rsx! {
        div {
            class: "relative",
            class: if props.disabled { "pointer-events-none opacity-50" },
            canvas {
                class: "block touch-none {props.class}",
                onmounted: move |event| {
                    #[cfg(feature = "web")]
                    {
                        use dioxus::web::WebEventExt;
                        use web_sys::wasm_bindgen::JsCast;
                        if let Ok(element) = event.as_web_event().clone().dyn_into() {
                            canvas.set(Some(Canvas::new(element)));
                        }
                    }
                    #[cfg(not(feature = "web"))]
                    let _ = event;
                },
                onpointerdown: move |event| with_canvas(Canvas::pointer_down, event),
                onpointermove: move |event| with_canvas(Canvas::pointer_move, event),
                onpointerup: move |event| {
                    with_canvas(Canvas::pointer_up, event);
                    emit_change();
                },
                onresize: move |_| {
                    if let Some(canvas) = canvas.write().as_mut() {
                        canvas.fit_to_element();
                    }
                },
            }
            div {
                class: "absolute bottom-3 left-3 flex items-center gap-2",
                role: "radiogroup",
                aria_label: "Ink colour",
                for choice in Ink::ALL {
                    button {
                        key: "{choice.name()}",
                        r#type: "button",
                        role: "radio",
                        aria_checked: ink() == choice,
                        title: choice.name(),
                        class: "size-4 rounded-full border-2 transition-transform hover:scale-110",
                        class: if ink() == choice { "border-text scale-110" } else { "border-transparent" },
                        style: "background: {choice.css()}",
                        onclick: move |_| {
                            ink.set(choice);
                            if let Some(canvas) = canvas.write().as_mut() {
                                canvas.set_ink(choice);
                            }
                        },
                    }
                }
            }
            div { class: "absolute right-3 bottom-3 flex gap-1.5",
                PadButton {
                    label: "undo",
                    onclick: move |_| {
                        if let Some(canvas) = canvas.write().as_mut() {
                            canvas.undo();
                        }
                        emit_change();
                    },
                }
                PadButton {
                    label: "clear",
                    onclick: move |_| {
                        if let Some(canvas) = canvas.write().as_mut() {
                            canvas.clear();
                        }
                        emit_change();
                    },
                }
            }
        }
    }
}

#[component]
fn PadButton(label: &'static str, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "label-mono rounded-sm border border-border-strong bg-surface px-2 py-0.5 hover:border-accent hover:text-accent",
            onclick: move |event| onclick.call(event),
            {label}
        }
    }
}

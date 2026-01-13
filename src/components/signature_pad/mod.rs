use canvas::Canvas;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use web_sys::HtmlCanvasElement;
use web_sys::wasm_bindgen::JsCast;
mod canvas;
mod point;
mod popup;
pub use popup::SignaturePopup;
mod stroke;
mod utils;
#[derive(Props, PartialEq, Debug, Clone)]
pub struct SignaturePadProps {
    #[props(default)]
    class: String,
    #[props(default)]
    container_class: String,
    #[props(default)]
    disabled: bool,
    #[props(default)]
    on_change: Option<EventHandler<Option<String>>>,
    #[props(default)]
    on_canvas_ready: Option<EventHandler<Canvas>>,
}
#[component]
pub fn SignaturePad(props: SignaturePadProps) -> Element {
    let mut canvas = use_signal(|| None::<Canvas>);
    let set_canvas = use_callback(move |event: MountedEvent| {
        let html_canvas = event
            .as_web_event()
            .clone()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        let canvas_ref = Canvas::new(html_canvas);
        canvas_ref.beautify();
        canvas.set(Some(canvas_ref.clone()));
        if let Some(on_canvas_ready) = &props.on_canvas_ready {
            on_canvas_ready.call(canvas_ref.clone());
        }
    });
    let on_signature_change = move || {
        if let Some(c) = canvas.read().as_ref() {
            let signature_data = c.get_signature_data();
            if let Some(on_change) = &props.on_change {
                on_change.call(Some(signature_data));
            }
        }
    };
    let on_pointer_down = move |event: PointerEvent| {
        if let Some(c) = canvas.read().as_ref() {
            c.on_mouse_down(&event);
        }
    };
    let on_pointer_move = move |event: PointerEvent| {
        if let Some(c) = canvas.read().as_ref() {
            c.on_mouse_move(&event);
        }
    };
    let on_pointer_up = move |event: PointerEvent| {
        if let Some(c) = canvas.read().as_ref() {
            c.on_mouse_up(&event);
            on_signature_change();
        }
    };
    let on_resize = move |_| {
        if let Some(c) = canvas.write().as_mut() {
            c.on_resize()
        }
    };
    rsx! {
        div {
            class: format!(
                "relative block {} {}",
                props.container_class,
                if props.disabled { "pointer-events-none opacity-50" } else { "" },
            ),
            canvas {
                onmounted: move |evt| set_canvas.call(evt),
                class: format!("relative block {}", props.class),
                style: "touch-action: none",
                onpointerdown: on_pointer_down,
                onpointermove: on_pointer_move,
                onpointerup: on_pointer_up,
                onresize: on_resize,
            }
            div { class: "absolute bottom-4 left-4 flex gap-2",
                button {
                    class: "font-sans text-sm bg-jet text-stone-300 px-2 py-1 rounded-md",
                    r#type: "button",
                    onclick: move |_| {
                        if let Some(c) = canvas.read().as_ref() {
                            c.undo();
                            on_signature_change();
                        }
                    },
                    "Undo"
                }
            }
            div { class: "absolute bottom-4 right-4 flex gap-2",
                button {
                    class: "font-sans text-sm bg-jet text-stone-300 px-2 py-1 rounded-md",
                    r#type: "button",
                    onclick: move |_| {
                        if let Some(c) = canvas.read().as_ref() {
                            c.clear();
                            on_signature_change();
                        }
                    },
                    "Clear"
                }
            }
        }
    }
}

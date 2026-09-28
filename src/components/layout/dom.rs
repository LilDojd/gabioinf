use dioxus::prelude::{Callback, Signal, WritableExt};
use web_sys::{
    Window,
    wasm_bindgen::{JsCast, closure::Closure},
};

pub(super) fn install_reading_progress(mut progress: Signal<f32>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if let Some(value) = current_scroll_progress(&window) {
        progress.set(value);
    }

    let scroll_window = window.clone();
    let on_scroll = Callback::new(move |()| {
        if let Some(value) = current_scroll_progress(&scroll_window) {
            progress.set(value);
        }
    });
    let listener = Closure::<dyn FnMut()>::new(move || on_scroll.call(()));

    if window
        .add_event_listener_with_callback("scroll", listener.as_ref().unchecked_ref())
        .is_ok()
    {
        // The layout and its window listener both live for the browser session.
        listener.forget();
    }
}

fn current_scroll_progress(window: &Window) -> Option<f32> {
    let document = window.document()?;
    let root = document.document_element()?;
    let viewport_height = window.inner_height().ok()?.as_f64()?;
    let scroll_y = window.scroll_y().ok()?;
    Some(super::scroll_progress(
        scroll_y,
        f64::from(root.scroll_height()),
        viewport_height,
    ))
}

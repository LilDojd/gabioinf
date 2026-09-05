use dioxus::{
    core::{Runtime, ScopeId},
    prelude::{Signal, WritableExt},
};
use std::rc::Rc;
use web_sys::{
    Window,
    wasm_bindgen::{JsCast, closure::Closure},
};

/// Browser callbacks run outside any Dioxus scope, so touching signals or the
/// navigator from them would panic; this re-enters the installing scope.
#[derive(Clone)]
pub(super) struct DioxusScope {
    runtime: Rc<Runtime>,
    scope: ScopeId,
}

impl DioxusScope {
    pub(super) fn current() -> Self {
        let runtime = Runtime::current();
        let scope = runtime.current_scope_id();
        Self { runtime, scope }
    }

    pub(super) fn enter<T>(&self, f: impl FnOnce() -> T) -> T {
        self.runtime.in_scope(self.scope, f)
    }
}

pub(super) fn install_reading_progress(mut progress: Signal<f32>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if let Some(value) = current_scroll_progress(&window) {
        progress.set(value);
    }

    let scope = DioxusScope::current();
    let scroll_window = window.clone();
    let listener = Closure::<dyn FnMut()>::new(move || {
        scope.enter(|| {
            if let Some(value) = current_scroll_progress(&scroll_window) {
                progress.set(value);
            }
        });
    });

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

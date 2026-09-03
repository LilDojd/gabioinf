//! Window-level keyboard shortcuts wired with web-sys: no JavaScript, just
//! `keydown`/`keyup` listeners feeding [`Chords`] and a `requestAnimationFrame`
//! loop for held `j`/`k` scrolling.

use super::{
    UiState,
    chords::{Action, Chords, Direction, Key},
    close_overlays,
    dom::DioxusScope,
    summon_sesh,
};
use dioxus::prelude::{WritableExt, dioxus_router::Navigator};
use std::{cell::RefCell, rc::Rc};
use web_sys::{
    HtmlElement, KeyboardEvent, ScrollBehavior, ScrollToOptions, Window,
    wasm_bindgen::{JsCast, JsValue, closure::Closure},
};

const TAP_MILLIS: f64 = 150.0;
const STEP_PIXELS: f64 = 80.0;
const HOLD_PIXELS_PER_SECOND: f64 = 1200.0;

#[derive(Clone, Copy)]
struct HeldScroll {
    direction: Direction,
    started: f64,
    last_frame: Option<f64>,
    frame_id: i32,
}

pub fn install(ui: UiState, navigator: Navigator) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let scope = DioxusScope::current();

    web_sys::console::log_2(
        &JsValue::from_str(
            "%c  .-\"\"\"-.\n /  o o  \\   hi, curious one.\n |   ^    |  the source is at github.com/LilDojd/gabioinf\n  \\  ---  /   try pressing / or ? on the page.\n   `-----´",
        ),
        &JsValue::from_str("color:#c2f9bb;font-family:monospace"),
    );

    let chords = Rc::new(RefCell::new(Chords::default()));
    let held = Rc::new(RefCell::new(None::<HeldScroll>));
    let animation = install_scroll_animation(&window, held.clone(), ui, scope.clone());

    let keydown = {
        let window = window.clone();
        let chords = chords.clone();
        let held = held.clone();
        let animation = animation.clone();
        let scope = scope.clone();
        Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
            scope.enter(|| {
                let key = event_key(&event);
                let typing = is_typing(&event);

                if matches!(key, Key::MetaK) || (matches!(key, Key::Slash) && !typing) {
                    event.prevent_default();
                }
                if event.repeat() && matches!(key, Key::Character('j' | 'k')) {
                    return;
                }
                if typing && !matches!(key, Key::Escape | Key::MetaK) {
                    return;
                }
                if overlays_open(ui) && !matches!(key, Key::Escape | Key::MetaK) {
                    return;
                }

                let action = {
                    let mut chords = chords.borrow_mut();
                    chords.now_millis = event.time_stamp();
                    chords.handle(&key)
                };
                let Some(action) = action else {
                    return;
                };
                match action {
                    Action::Scroll { direction, count } => {
                        if let Some(count) = count {
                            stop_hold(&window, &held, false);
                            scroll(&window, direction, f64::from(count) * STEP_PIXELS, true);
                        } else {
                            start_hold(&window, &held, &animation, direction, event.time_stamp());
                        }
                    }
                    action => perform(action, ui, navigator),
                }
            })
        })
    };

    let keyup = {
        let window = window.clone();
        let held = held.clone();
        let scope = scope.clone();
        Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
            scope.enter(|| {
                let direction = match event.key().to_ascii_lowercase().as_str() {
                    "j" => Direction::Down,
                    "k" => Direction::Up,
                    _ => return,
                };
                let short_tap = held.borrow().is_some_and(|active| {
                    active.direction == direction
                        && event.time_stamp() - active.started < TAP_MILLIS
                        && !overlays_open(ui)
                });
                if held
                    .borrow()
                    .is_some_and(|active| active.direction == direction)
                {
                    stop_hold(&window, &held, short_tap);
                }
            })
        })
    };

    if window
        .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
        .is_ok()
    {
        // Layout lives for the session, so leaking these listener closures is intentional.
        keydown.forget();
    }
    if window
        .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())
        .is_ok()
    {
        keyup.forget();
    }

    // The self-scheduling animation callback intentionally shares the layout's lifetime.
    drop(animation);
}

type AnimationCallback = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

fn install_scroll_animation(
    window: &Window,
    held: Rc<RefCell<Option<HeldScroll>>>,
    ui: UiState,
    scope: DioxusScope,
) -> AnimationCallback {
    let animation = AnimationCallback::default();
    let callback_slot = animation.clone();
    let frame_window = window.clone();
    let frame_held = held.clone();

    *animation.borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        scope.enter(|| {
            let mut active_scroll = frame_held.borrow_mut();
            let Some(active) = active_scroll.as_mut() else {
                return;
            };
            if overlays_open(ui) {
                *active_scroll = None;
                return;
            }

            let elapsed = timestamp - active.started;
            if elapsed >= TAP_MILLIS {
                if let Some(last_frame) = active.last_frame {
                    let seconds = ((timestamp - last_frame).min(50.0)) / 1000.0;
                    scroll(
                        &frame_window,
                        active.direction,
                        HOLD_PIXELS_PER_SECOND * seconds,
                        false,
                    );
                }
                active.last_frame = Some(timestamp);
            }

            if let Some(callback) = callback_slot.borrow().as_ref()
                && let Ok(frame_id) =
                    frame_window.request_animation_frame(callback.as_ref().unchecked_ref())
            {
                active.frame_id = frame_id;
            }
        })
    }));

    animation
}

fn start_hold(
    window: &Window,
    held: &Rc<RefCell<Option<HeldScroll>>>,
    animation: &AnimationCallback,
    direction: Direction,
    started: f64,
) {
    stop_hold(window, held, false);
    let animation = animation.borrow();
    let Some(callback) = animation.as_ref() else {
        return;
    };
    let Ok(frame_id) = window.request_animation_frame(callback.as_ref().unchecked_ref()) else {
        return;
    };
    *held.borrow_mut() = Some(HeldScroll {
        direction,
        started,
        last_frame: None,
        frame_id,
    });
}

fn stop_hold(window: &Window, held: &Rc<RefCell<Option<HeldScroll>>>, single_step: bool) {
    let Some(active) = held.borrow_mut().take() else {
        return;
    };
    let _ = window.cancel_animation_frame(active.frame_id);
    if single_step {
        scroll(window, active.direction, STEP_PIXELS, true);
    }
}

fn scroll(window: &Window, direction: Direction, pixels: f64, smooth: bool) {
    let options = ScrollToOptions::new();
    let signed_pixels = match direction {
        Direction::Down => pixels,
        Direction::Up => -pixels,
    };
    options.set_top(signed_pixels);
    options.set_behavior(if smooth {
        ScrollBehavior::Smooth
    } else {
        ScrollBehavior::Auto
    });
    window.scroll_by_with_scroll_to_options(&options);
}

fn perform(action: Action, mut ui: UiState, navigator: Navigator) {
    match action {
        Action::Navigate(route) => {
            navigator.push(route);
            close_overlays(ui);
        }
        Action::TogglePalette => {
            let open = !(ui.palette_open)();
            ui.palette_open.set(open);
            ui.help_open.set(false);
        }
        Action::OpenPalette => {
            ui.palette_open.set(true);
            ui.help_open.set(false);
        }
        Action::ToggleHelp => {
            let open = !(ui.help_open)();
            ui.help_open.set(open);
            ui.palette_open.set(false);
        }
        Action::CloseOverlays => close_overlays(ui),
        Action::Sesh => summon_sesh(ui),
        Action::Scroll { .. } => {}
    }
}

fn event_key(event: &KeyboardEvent) -> Key {
    let key = event.key().to_ascii_lowercase();
    if (event.meta_key() || event.ctrl_key()) && key == "k" {
        return Key::MetaK;
    }
    if event.meta_key() || event.ctrl_key() || event.alt_key() {
        return Key::Other;
    }
    match key.as_str() {
        "escape" => Key::Escape,
        "/" => Key::Slash,
        "?" => Key::Question,
        _ => {
            let mut characters = key.chars();
            match (characters.next(), characters.next()) {
                (Some(character), None) => Key::Character(character),
                _ => Key::Other,
            }
        }
    }
}

fn is_typing(event: &KeyboardEvent) -> bool {
    event.target().is_some_and(|target| {
        target.dyn_ref::<HtmlElement>().is_some_and(|element| {
            matches!(element.tag_name().as_str(), "INPUT" | "TEXTAREA")
                || element.is_content_editable()
        })
    })
}

fn overlays_open(ui: UiState) -> bool {
    (ui.palette_open)() || (ui.help_open)()
}

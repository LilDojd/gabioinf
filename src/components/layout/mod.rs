use crate::Route;
use dioxus::prelude::*;
use dioxus_router::{Navigator, components::Outlet};
#[cfg(feature = "web")]
use serde::Deserialize;
use std::time::Duration;
use time::UtcOffset;
#[cfg(feature = "web")]
use wasmtimer::std::Instant;
use wasmtimer::tokio::sleep;

mod footer;
mod navbar;
use navbar::{MobileFooter, Sidebar};

static CV: Asset = asset!("/assets/CV_GeorgyAndreev_042025.pdf");

#[cfg(feature = "web")]
const KEYBOARD_SCRIPT: &str = r#"
const handler = (event) => {
    const target = event.target;
    const typing = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target?.isContentEditable;
    const meta = event.metaKey || event.ctrlKey;
    if ((meta && event.key.toLowerCase() === 'k') || event.key === '/') event.preventDefault();
    dioxus.send({ key: event.key, meta, typing });
};
window.addEventListener('keydown', handler);
console.log('%c  .-"""-.\n /  o o  \\   hi, curious one.\n |   ^    |  the source is at github.com/LilDojd/gabioinf\n  \\  ---  /   try pressing / or ? on the page.\n   `-----´', 'color:#c2f9bb;font-family:monospace');
await new Promise(() => {});
"#;

#[derive(Clone, Copy)]
pub(crate) struct UiState {
    pub palette_open: Signal<bool>,
    pub help_open: Signal<bool>,
    pub sesh_visible: Signal<bool>,
    pub retype: Signal<u32>,
}

#[cfg(feature = "web")]
#[derive(Deserialize)]
struct KeyStroke {
    key: String,
    meta: bool,
    typing: bool,
}

#[component]
pub fn Layout() -> Element {
    let ui = UiState {
        palette_open: use_signal(|| false),
        help_open: use_signal(|| false),
        sesh_visible: use_signal(|| false),
        retype: use_signal(|| 0),
    };
    use_context_provider(move || ui);

    let clock = use_signal(abu_dhabi_time);
    #[cfg(feature = "web")]
    {
        let mut clock_update = clock;
        use_effect(move || {
            spawn(async move {
                loop {
                    sleep(Duration::from_secs(30)).await;
                    clock_update.set(abu_dhabi_time());
                }
            });
        });
    }

    #[cfg(feature = "web")]
    let navigator = navigator();
    #[cfg(feature = "web")]
    use_effect(move || {
        let mut ui = ui;
        spawn(async move {
            let mut eval = document::eval(KEYBOARD_SCRIPT);
            let mut pending_g = None::<Instant>;
            let mut buffer = String::new();
            while let Ok(event) = eval.recv::<KeyStroke>().await {
                if event.key == "Escape" {
                    close_overlays(ui);
                    continue;
                }
                if event.meta && event.key.eq_ignore_ascii_case("k") {
                    let open = !(ui.palette_open)();
                    ui.palette_open.set(open);
                    ui.help_open.set(false);
                    continue;
                }
                if event.typing {
                    continue;
                }
                match event.key.as_str() {
                    "/" => {
                        ui.palette_open.set(true);
                        ui.help_open.set(false);
                        continue;
                    }
                    "?" => {
                        let open = !(ui.help_open)();
                        ui.help_open.set(open);
                        ui.palette_open.set(false);
                        continue;
                    }
                    "j" => {
                        _ = document::eval("window.scrollBy({ top: 80, behavior: 'smooth' })");
                    }
                    "k" => {
                        _ = document::eval("window.scrollBy({ top: -80, behavior: 'smooth' })");
                    }
                    _ => {}
                }

                if pending_g.is_some_and(|started| started.elapsed() <= Duration::from_millis(900))
                {
                    pending_g = None;
                    if let Some(page) = Page::from_key(&event.key) {
                        navigate(page, navigator);
                        close_overlays(ui);
                        continue;
                    }
                } else {
                    pending_g = None;
                }
                if event.key == "g" {
                    pending_g = Some(Instant::now());
                }
                if event.key.chars().count() == 1 {
                    buffer.push_str(&event.key.to_lowercase());
                    if buffer.len() > 4 {
                        buffer.remove(0);
                    }
                    if buffer == "sesh" {
                        buffer.clear();
                        summon_sesh(ui);
                    }
                }
            }
        });
    });

    let route: Route = use_route();
    rsx! {
        crate::DocumentMetadata {}
        div { class: "grid min-h-screen grid-cols-1 justify-center px-5 min-[760px]:grid-cols-[220px_minmax(0,600px)] min-[760px]:gap-[72px] min-[760px]:px-8",
            Sidebar { clock: clock() }
            main { class: "min-w-0 py-8 pb-14 min-[760px]:py-12 min-[760px]:pb-24",
                Outlet::<Route> {}
                div { class: "mt-16",
                    Link { to: Route::NotFound { route: vec!["void".to_string()] }, class: "label-mono text-faint no-underline hover:text-accent", "/404" }
                }
            }
            MobileFooter { clock: clock() }
            if matches!(route, Route::BlogPost { .. }) {
                div { class: "reading-progress fixed top-0 left-0 z-20 h-0.5 w-full bg-accent" }
            }
            img {
                class: if (ui.sesh_visible)() { "fixed right-6 bottom-0 z-30 w-[150px] rounded-t-md shadow-[0_-8px_30px_rgba(0,0,0,.4)] transition-[bottom] duration-500 [transition-timing-function:cubic-bezier(.2,.8,.2,1)] pointer-events-none" } else { "fixed right-6 -bottom-40 z-30 w-[150px] rounded-t-md shadow-[0_-8px_30px_rgba(0,0,0,.4)] transition-[bottom] duration-500 [transition-timing-function:cubic-bezier(.2,.8,.2,1)] pointer-events-none" },
                src: asset!("/assets/sesh.avif"),
                alt: "Sesh peeking",
            }
            if (ui.palette_open)() { CommandPalette {} }
            if (ui.help_open)() { HelpSheet {} }
        }
    }
}

fn abu_dhabi_time() -> String {
    let offset = UtcOffset::from_hms(4, 0, 0).expect("UTC+4 is a valid fixed offset");
    let now = time::OffsetDateTime::now_utc().to_offset(offset);
    format!("{:02}:{:02}", now.hour(), now.minute())
}

fn close_overlays(mut ui: UiState) {
    ui.palette_open.set(false);
    ui.help_open.set(false);
}

fn summon_sesh(mut ui: UiState) {
    ui.palette_open.set(false);
    ui.sesh_visible.set(true);
    spawn(async move {
        sleep(Duration::from_millis(3200)).await;
        ui.sesh_visible.set(false);
    });
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Page {
    Home,
    Blog,
    Projects,
    About,
    Guestbook,
    Void,
}

impl Page {
    #[cfg(any(feature = "web", test))]
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "h" => Some(Self::Home),
            "b" => Some(Self::Blog),
            "p" => Some(Self::Projects),
            "a" => Some(Self::About),
            "g" => Some(Self::Guestbook),
            "v" => Some(Self::Void),
            _ => None,
        }
    }
}

fn navigate(page: Page, navigator: Navigator) {
    match page {
        Page::Home => navigator.push(Route::Home {}),
        Page::Blog => navigator.push(Route::Blog {}),
        Page::Projects => navigator.push(Route::Projects {}),
        Page::About => navigator.push(Route::AboutMe {}),
        Page::Guestbook => navigator.push(Route::Guestbook {}),
        Page::Void => navigator.push(Route::NotFound {
            route: vec!["void".to_string()],
        }),
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommandTarget {
    Page(Page),
    External(&'static str),
    Cv,
    Retype,
    Sesh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Command {
    label: &'static str,
    hint: &'static str,
    target: CommandTarget,
}

const COMMANDS: &[Command] = &[
    Command {
        label: "home",
        hint: "g h",
        target: CommandTarget::Page(Page::Home),
    },
    Command {
        label: "blog",
        hint: "g b",
        target: CommandTarget::Page(Page::Blog),
    },
    Command {
        label: "projects",
        hint: "g p",
        target: CommandTarget::Page(Page::Projects),
    },
    Command {
        label: "about me",
        hint: "g a",
        target: CommandTarget::Page(Page::About),
    },
    Command {
        label: "guestbook",
        hint: "g g",
        target: CommandTarget::Page(Page::Guestbook),
    },
    Command {
        label: "the void",
        hint: "g v",
        target: CommandTarget::Page(Page::Void),
    },
    Command {
        label: "github",
        hint: "↗",
        target: CommandTarget::External("https://github.com/LilDojd"),
    },
    Command {
        label: "linkedin",
        hint: "↗",
        target: CommandTarget::External("https://www.linkedin.com/in/georgiy-andreev"),
    },
    Command {
        label: "book a chat",
        hint: "↗",
        target: CommandTarget::External("https://cal.com/yawner"),
    },
    Command {
        label: "cv (pdf)",
        hint: "↗",
        target: CommandTarget::Cv,
    },
    Command {
        label: "email",
        hint: "↗",
        target: CommandTarget::External("mailto:yawner@pm.me"),
    },
    Command {
        label: "retype greeting",
        hint: "",
        target: CommandTarget::Retype,
    },
    Command {
        label: "summon sesh",
        hint: "",
        target: CommandTarget::Sesh,
    },
];

#[component]
fn CommandPalette() -> Element {
    let ui = use_context::<UiState>();
    let navigator = navigator();
    let mut query = use_signal(String::new);
    let normalized = query().trim().to_lowercase();
    let results = COMMANDS
        .iter()
        .copied()
        .filter(|command| normalized.is_empty() || command.label.contains(&normalized))
        .collect::<Vec<_>>();
    let first = results.first().copied();
    let no_results = results.is_empty();

    rsx! {
        div {
            class: "fixed inset-0 z-40 flex items-start justify-center bg-[rgba(10,11,13,.6)] pt-[18vh] backdrop-blur-sm",
            onclick: move |_| close_overlays(ui),
            div {
                class: "w-[440px] max-w-[calc(100%-48px)] overflow-hidden rounded-lg border border-border-strong bg-surface shadow-[0_20px_60px_rgba(0,0,0,.5)]",
                onclick: move |event| event.stop_propagation(),
                input {
                    autofocus: true,
                    class: "w-full border-0 border-b border-card bg-transparent px-4 py-3.5 text-base text-text outline-none placeholder:text-label [font-variation-settings:'MONO'_0,'CASL'_.5,'wght'_400]",
                    placeholder: "where to?",
                    value: query,
                    oninput: move |event| query.set(event.value()),
                    onkeydown: move |event| {
                        if event.key() == Key::Enter
                            && let Some(command) = first
                        {
                            run_command(command.target, navigator, ui);
                        }
                    },
                }
                div { class: "flex max-h-80 flex-col overflow-y-auto p-1.5",
                    for (index, command) in results.into_iter().enumerate() {
                        button {
                            key: "{command.label}",
                            r#type: "button",
                            class: if index == 0 { "flex w-full items-baseline justify-between gap-4 rounded-sm bg-hover-row px-2.5 py-2 text-left text-sm text-accent" } else { "flex w-full items-baseline justify-between gap-4 rounded-sm bg-transparent px-2.5 py-2 text-left text-sm text-secondary hover:bg-hover-row hover:text-accent" },
                            onclick: move |_| run_command(command.target, navigator, ui),
                            span { "{command.label}" }
                            span { class: "label-mono text-[11px]", "{command.hint}" }
                        }
                    }
                    if no_results {
                        span { class: "label-mono px-2.5 py-3", "nothing out here" }
                    }
                }
            }
        }
    }
}

fn run_command(target: CommandTarget, navigator: Navigator, mut ui: UiState) {
    match target {
        CommandTarget::Page(page) => navigate(page, navigator),
        CommandTarget::External(url) => {
            #[cfg(feature = "web")]
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url_and_target_and_features(url, "_blank", "noopener");
            }
            #[cfg(not(feature = "web"))]
            let _ = url;
        }
        CommandTarget::Cv => {
            let url = CV.to_string();
            #[cfg(feature = "web")]
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url_and_target_and_features(&url, "_blank", "noopener");
            }
            #[cfg(not(feature = "web"))]
            let _ = url;
        }
        CommandTarget::Retype => {
            navigator.push(Route::Home {});
            let next = (ui.retype)().wrapping_add(1);
            ui.retype.set(next);
        }
        CommandTarget::Sesh => summon_sesh(ui),
    }
    ui.palette_open.set(false);
}

#[component]
fn HelpSheet() -> Element {
    let ui = use_context::<UiState>();
    rsx! {
        div {
            class: "fixed inset-0 z-40 flex items-center justify-center bg-[rgba(10,11,13,.6)] backdrop-blur-sm",
            onclick: move |_| close_overlays(ui),
            div {
                class: "grid w-[360px] max-w-[calc(100%-48px)] grid-cols-[auto_1fr] gap-x-5 gap-y-2.5 rounded-lg border border-border-strong bg-surface px-[22px] py-5 text-[13px] text-secondary shadow-[0_20px_60px_rgba(0,0,0,.5)]",
                onclick: move |event| event.stop_propagation(),
                span { class: "label-mono col-span-2 mb-1", "// keys" }
                kbd { class: "text-accent [font-variation-settings:'MONO'_1]", "/" } span { "jump anywhere" }
                kbd { class: "text-accent [font-variation-settings:'MONO'_1]", "g h · b · p · a · g" } span { "home, blog, projects, about, guestbook" }
                kbd { class: "text-accent [font-variation-settings:'MONO'_1]", "j / k" } span { "scroll, the neovim way" }
                kbd { class: "text-accent [font-variation-settings:'MONO'_1]", "?" } span { "this" }
                kbd { class: "text-accent [font-variation-settings:'MONO'_1]", "esc" } span { "close" }
                span { class: "label-mono col-span-2 mt-2 text-faint", "there are a few more. the cat knows." }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_navigation_chords() {
        assert_eq!(Page::from_key("h"), Some(Page::Home));
        assert_eq!(Page::from_key("v"), Some(Page::Void));
        assert_eq!(Page::from_key("x"), None);
    }
}

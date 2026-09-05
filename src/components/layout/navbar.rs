use super::footer::AreciboFooter;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Sidebar(clock: String) -> Element {
    rsx! {
        aside { class: "flex flex-wrap items-baseline justify-between gap-x-5 gap-y-3 py-6 pb-2 md:sticky md:top-0 md:h-screen md:flex-col md:items-stretch md:gap-7 md:overflow-y-auto md:py-10 md:pb-6 [scrollbar-width:none]",
            Wordmark {}
            Navigation {}
            div { class: "hidden md:mt-auto md:flex md:flex-col md:gap-7",
                Status { clock: clock.clone(), show_hint: true }
                AreciboFooter {}
            }
        }
    }
}

#[component]
pub fn MobileFooter(clock: String) -> Element {
    rsx! {
        footer { class: "flex flex-wrap items-end justify-between gap-6 border-t border-line py-5 pb-8 md:hidden",
            Status { clock, show_hint: false }
            div { class: "min-w-[220px] grow sm:grow-0", AreciboFooter {} }
        }
    }
}

#[component]
fn Wordmark() -> Element {
    rsx! {
        Link { to: Route::Home {}, class: "flex flex-col gap-0.5 text-text no-underline",
            span { class: "casual text-base", "george andreev" }
            span { class: "label-mono", "gabioinf.dev" }
        }
    }
}

#[component]
fn Navigation() -> Element {
    let route: Route = use_route();
    rsx! {
        nav { aria_label: "Main navigation", class: "flex flex-row flex-wrap gap-x-[14px] gap-y-1 text-[15px] md:flex-col md:gap-1.5",
            NavItem { to: Route::Home {}, label: "home", active: matches!(route, Route::Home {}) }
            NavItem { to: Route::Blog {}, label: "blog", active: matches!(route, Route::Blog {} | Route::BlogPost { .. }) }
            NavItem { to: Route::Projects {}, label: "projects", active: matches!(route, Route::Projects {}) }
            NavItem { to: Route::AboutMe {}, label: "about me", active: matches!(route, Route::AboutMe {}) }
            NavItem { to: Route::Guestbook {}, label: "guestbook", active: matches!(route, Route::Guestbook {}) }
        }
    }
}

#[component]
fn NavItem(to: Route, label: &'static str, active: bool) -> Element {
    rsx! {
        Link {
            to,
            class: if active { "nav-item-active" } else { "nav-item" },
            span { class: if active { "nav-dot size-1.5 shrink-0 rounded-full bg-accent" } else { "nav-dot size-1.5 shrink-0 rounded-full bg-border-strong" } }
            {label}
        }
    }
}

#[component]
fn Status(clock: String, show_hint: bool) -> Element {
    rsx! {
        div { class: "label-mono flex flex-col gap-1.5 leading-[1.5]",
            span {
                "research engineer @ "
                a { href: "https://genbio.ai/", target: "_blank", rel: "noopener noreferrer", class: "text-muted no-underline hover:text-accent", "genbio ai" }
            }
            span { "abu dhabi, uae · {clock}" }
            if show_hint {
                span { class: "mt-1 text-faint",
                    "press "
                    kbd { class: "rounded-[3px] border border-border-strong px-1 font-inherit text-label", "/" }
                    " to jump · "
                    kbd { class: "rounded-[3px] border border-border-strong px-1 font-inherit text-label", "?" }
                    " for keys"
                }
            }
        }
    }
}

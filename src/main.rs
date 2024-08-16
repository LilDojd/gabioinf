#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use tracing::Level;
#[cfg(feature = "server")]
mod backend;
mod components;
mod markdown;
mod pages;

use components::layout::NavFooter;
use pages::{AboutMe, Blog, Guestbook, Home, NotFound, Projects};
const TAILWIND: &str = asset!("assets/tailwind.css");
const STYLE: &str = asset!("assets/main.css");
const NAVBAR: &str = asset!("assets/navbar.css");
const LINKS: &str = asset!("assets/alien_links.css");

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("Starting server");

    LaunchBuilder::fullstack()
        .with_cfg(server_only!(ServeConfig::builder().incremental(
            IncrementalRendererConfig::default()
                .invalidate_after(std::time::Duration::from_secs(120)),
        )))
        .launch(App);
}

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[layout(NavFooter)]
    #[route("/")]
    Home {},
    #[route("/blog")]
    Blog {},
    #[route("/projects")]
    Projects {},
    #[route("/about")]
    AboutMe {},
    #[route("/guestbook")]
    Guestbook {},
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}
fn App() -> Element {
    rsx! {
        head::Link { rel: "stylesheet", href: TAILWIND }
        head::Link { rel: "stylesheet", href: STYLE }
        head::Link { rel: "stylesheet", href: NAVBAR }
        head::Link { rel: "stylesheet", href: LINKS }
        head::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Recursive:slnt,wght,CASL,CRSV,MONO@-15..0,300..800,0..1,0..1,0..1&display=swap",
        }
        Router::<Route> {}
    }
}

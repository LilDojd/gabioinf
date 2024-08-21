#![allow(non_snake_case)]
use dioxus::prelude::*;
use shared::models::GuestbookEntry;
use std::str::FromStr;
use tracing::Level;
#[cfg(feature = "server")]
mod backend;
mod components;
mod hide;
mod markdown;
mod pages;
mod shared;
use components::layout::NavFooter;
use pages::{AboutMe, Blog, Guestbook, Home, NotFound, Projects};
const TAILWIND: &str = asset!("public/tailwind.css");
const STYLE: &str = asset!("public/main.css");
const NAVBAR: &str = asset!("public/navbar.css");
const LINKS: &str = asset!("public/alien_links.css");
#[derive(Clone, Debug)]
pub struct MessageValid(bool, String);
fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    dioxus_logger::init(Level::from_str(&log_level).unwrap_or(Level::INFO))
        .expect("failed to init logger");
    #[cfg(feature = "web")]
    dioxus_web::launch::launch_cfg(App, dioxus_web::Config::new().hydrate(true));
    #[cfg(feature = "server")]
    {
        let _guard = sentry::init(sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        });
        dioxus_logger::tracing::info!("Starting server");
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                backend::server::serve(ServeConfig::new().unwrap(), App).await
            });
    }
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
    use_context_provider(|| Signal::new(MessageValid(true, String::new())));
    use_context_provider(|| Signal::new(None::<GuestbookEntry>));
    rsx! {
        head::Link { rel: "stylesheet", href: TAILWIND }
        head::Link { rel: "stylesheet", href: STYLE }
        head::Link { rel: "stylesheet", href: NAVBAR }
        head::Link { rel: "stylesheet", href: LINKS }
        head::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Recursive:slnt,wght,CASL@-15..0,300..800,0..1&display=swap",
        }
        ErrorBoundary {
            handle_error: |errors: ErrorContext| {
                match errors.show() {
                    Some(view) => view,
                    None => rsx! {
                        pre { color: "#ef6f6c", "oops, we ran into an error\n{errors:#?}" }
                    },
                }
            },
            Router::<Route> {}
        }
    }
}

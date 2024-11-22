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
            .block_on(
                async move { backend::server::serve(ServeConfig::new().unwrap(), App).await },
            );
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
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Meta { charset: "UTF-8" }
        // Open Graph
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:title", content: "George Andreev personal website" }
        document::Meta {
            property: "og:description",
            content: "Personal website of George Andreev, bioinformatician and developer. Explore projects, blog posts, and sign the guestbook.",
        }
        document::Meta { property: "og:url", content: "https://gabioinf.dev" }

        document::Meta {
            property: "og:image",
            content: "https://github.com/LilDojd/gabioinf/blob/main/public/og-img.png?raw=true",
        }
        document::Meta { property: "og:image:width", content: "1200" }
        document::Meta { property: "og:image:height", content: "630" }

        // Twitter/X
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:title", content: "George Andreev personal website" }
        document::Meta {
            name: "twitter:description",
            content: "Personal website of George Andreev, bioinformatician and developer. Explore projects, blog posts, and sign the guestbook.",
        }
        document::Meta {
            name: "twitter:image",
            content: "https://github.com/LilDojd/gabioinf/blob/main/public/og-img.png?raw=true",
        }
        document::Meta { name: "twitter:image:width", content: "1200" }
        document::Meta { name: "twitter:image:height", content: "630" }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "true",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Recursive:slnt,wght,CASL@-15..0,300..800,0..1&display=swap",
        }
        document::Link { rel: "stylesheet", href: asset!("/public/tailwind.css") }
        document::Link { rel: "stylesheet", href: asset!("/public/alien_links.css") }
        document::Link { rel: "stylesheet", href: asset!("/public/main.css") }
        document::Link { rel: "stylesheet", href: asset!("/public/navbar.css") }
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

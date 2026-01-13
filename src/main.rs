#![allow(non_snake_case)]
use dioxus::prelude::*;
use shared::server_fns;
use std::str::FromStr;
use tracing::Level;
mod auth;
#[cfg(feature = "server")]
mod backend;
mod components;
mod hide;
mod markdown;
mod pages;
mod shared;
use auth::AuthState;
use components::layout::NavFooter;
use pages::{AboutMe, Blog, Guestbook, Home, NotFound, Projects};

static STYLES: Asset = asset!("/assets/styles");

#[derive(Clone, Debug)]
pub struct MessageValid(bool, String);

fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    dioxus_logger::init(Level::from_str(&log_level).unwrap_or(Level::INFO))
        .expect("failed to init logger");
    #[cfg(not(feature = "server"))]
    LaunchBuilder::new()
        .with_cfg(web! {
            dioxus::web::Config::new().hydrate(true)
        })
        .launch(App);
    #[cfg(feature = "server")]
    {
        let _guard = sentry::init(sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        });
        dioxus_logger::tracing::info!("Starting server");
        let config = ServeConfig::builder()
            .incremental(IncrementalRendererConfig::new())
            .enable_out_of_order_streaming()
            .build();
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move { backend::server::serve(config.unwrap(), App).await });
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
    use_context_provider(|| Signal::new(AuthState::Loading));
    use_context_provider(|| Signal::new(MessageValid(true, String::new())));

    let mut auth_state = use_context::<Signal<AuthState>>();

    use_effect(move || {
        spawn(async move {
            if let Ok(user_result) = server_fns::get_user().await {
                match user_result {
                    Some(user) => {
                        dioxus_logger::tracing::debug!(
                            "Fetching user signature for authenticated user"
                        );

                        let signature = match server_fns::load_user_signature(user.clone()).await {
                            Ok(signature) => signature,
                            Err(e) => {
                                dioxus_logger::tracing::error!(
                                    "Failed to load user signature: {:?}",
                                    e
                                );
                                None
                            }
                        };

                        let user_state = auth::UserState {
                            guest: user,
                            entry: signature,
                        };

                        auth_state.set(AuthState::Authenticated(Box::new(user_state)));
                    }
                    None => {
                        auth_state.set(AuthState::Unauthenticated);
                    }
                }
            }
        });
    });
    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Meta { charset: "UTF-8" }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:title", content: "George Andreev personal website" }
        document::Meta {
            property: "og:description",
            content: "Personal website of George Andreev, bioinformatician and developer. Explore projects, blog posts, and sign the guestbook.",
        }
        document::Meta { property: "og:url", content: "https://gabioinf.dev" }
        document::Meta {
            property: "og:image",
            content: "https://github.com/LilDojd/gabioinf/blob/main/assets/og-img.png?raw=true",
        }
        document::Meta { property: "og:image:width", content: "1200" }
        document::Meta { property: "og:image:height", content: "630" }
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:title", content: "George Andreev personal website" }
        document::Meta {
            name: "twitter:description",
            content: "Personal website of George Andreev, bioinformatician and developer. Explore projects, blog posts, and sign the guestbook.",
        }
        document::Meta {
            name: "twitter:image",
            content: "https://github.com/LilDojd/gabioinf/blob/main/assets/og-img.png?raw=true",
        }
        document::Meta { name: "twitter:image:width", content: "1200" }
        document::Meta { name: "twitter:image:height", content: "630" }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Stylesheet { href: asset!("assets/tailwind.css") }
        document::Stylesheet { href: "{STYLES}/alien_links.css" }
        document::Stylesheet { href: "{STYLES}/main.css" }
        document::Stylesheet { href: "{STYLES}/navbar.css" }
        ErrorBoundary {
            handle_error: |errors: ErrorContext| {
                let error = errors.error();
                rsx! {
                    div { class: "container mx-auto px-4 py-8",
                        article { class: "prose prose-invert prose-stone prose-h2:mb-0 lg:prose-lg mb-8",
                            h1 { class: "text-3xl font-bold mb-6", "Error" }
                            p { class: "text-lg", "An error occurred." }
                            p { class: "text-lg",
                                code { class: "text-red-500", "{error:?}" }
                            }
                            p { class: "text-lg",
                                "If you think this is a mistake, please "
                                a {
                                    href: "https://github.com/LilDojd/gabioinf/issues/new",
                                    target: "_blank",
                                    "open an issue on GitHub"
                                }
                                "."
                            }
                        }
                    }
                }
            },
            Router::<Route> {}
        }
    }
}

#![allow(non_snake_case, unreachable_code)]
use dioxus::fullstack::FullstackContext;
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
fn main() -> anyhow::Result<()> {
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
        let _guard =
            sentry::init(sentry::ClientOptions::new().maybe_release(sentry::release_name!()));
        dioxus_logger::tracing::info!("Starting server");
        // Dioxus 0.7.10 incremental cache hits currently lose custom route statuses.
        let config = ServeConfig::new().enable_out_of_order_streaming();
        tokio::runtime::Runtime::new()?.block_on(backend::server::serve(config, App))?;
    }
    Ok(())
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct RouteMetadata {
    title: &'static str,
    description: &'static str,
}

impl Route {
    fn metadata(&self) -> RouteMetadata {
        match self {
            Self::Home {} => RouteMetadata {
                title: "George Andreev | Bioinformatician and Developer",
                description: "George Andreev's personal website about bioinformatics, software development, projects, and experiments.",
            },
            Self::Blog {} => RouteMetadata {
                title: "Blog | George Andreev",
                description: "Notes and articles from George Andreev about bioinformatics, software development, and other experiments.",
            },
            Self::Projects {} => RouteMetadata {
                title: "Projects | George Andreev",
                description: "Selected software projects, publications, and professional milestones from George Andreev.",
            },
            Self::AboutMe {} => RouteMetadata {
                title: "About | George Andreev",
                description: "Learn about George Andreev, a bioinformatician and software developer working on machine learning for biology.",
            },
            Self::Guestbook {} => RouteMetadata {
                title: "Guestbook | George Andreev",
                description: "Read messages from visitors and sign George Andreev's guestbook with GitHub.",
            },
            Self::NotFound { .. } => RouteMetadata {
                title: "Page not found | George Andreev",
                description: "The requested page could not be found on George Andreev's website.",
            },
        }
    }
}

fn DocumentMetadata() -> Element {
    let router = router();
    let route: Route = router.current();
    let metadata = route.metadata();
    let initial_metadata = use_hook(|| metadata);

    use_effect(move || {
        #[cfg(feature = "web")]
        {
            let metadata = router.current::<Route>().metadata();
            let description = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.get_element_by_id("route-description"));

            if let Some(description) = description {
                _ = description.set_attribute("content", metadata.description);
            }
        }
    });

    rsx! {
        document::Title { "{metadata.title}" }
        document::Meta {
            id: "route-description",
            name: "description",
            content: initial_metadata.description,
        }
    }
}

fn App() -> Element {
    use_context_provider(|| Signal::new(MessageValid(true, String::new())));
    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Meta { charset: "UTF-8" }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Stylesheet { href: asset!("assets/tailwind.css") }
        document::Stylesheet { href: "{STYLES}/alien_links.css" }
        document::Stylesheet { href: "{STYLES}/main.css" }
        document::Stylesheet { href: "{STYLES}/navbar.css" }
        ErrorBoundary {
            handle_error: render_error,
            AppRouter {}
        }
    }
}

#[component]
fn AppRouter() -> Element {
    let auth_state = use_loader(|| async {
        let Some(user) = server_fns::get_user().await? else {
            return Ok::<_, ServerFnError>(AuthState::Unauthenticated);
        };
        let entry = server_fns::load_user_signature(user.clone()).await?;
        Ok(AuthState::Authenticated(Box::new(auth::UserState {
            guest: user,
            entry,
        })))
    })?;
    use_context_provider(|| auth_state);

    rsx! { Router::<Route> {} }
}

fn render_error(errors: ErrorContext) -> Element {
    let error = errors
        .error()
        .expect("error boundary must contain an error");
    FullstackContext::commit_error_status(error.clone());

    rsx! {
        div { class: "container mx-auto px-4 py-8",
            article { class: "prose prose-invert prose-stone prose-h2:mb-0 lg:prose-lg mb-8",
                h1 { class: "text-3xl font-bold mb-6", "Error" }
                p { class: "text-lg", "An error occurred." }
                p { class: "text-lg",
                    code { class: "text-red-500", "{error}" }
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
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use axum::{body::Body, extract::State, http::Request};
    use dioxus::server::FullstackState;

    async fn render_status(app: fn() -> Element, uri: &str) -> StatusCode {
        let state = FullstackState::new(ServeConfig::new(), app);
        let request = Request::builder().uri(uri).body(Body::empty()).unwrap();

        FullstackState::render_handler(State(state), request)
            .await
            .status()
    }

    #[tokio::test]
    async fn not_found_route_returns_404() {
        assert_eq!(render_status(App, "/missing").await, StatusCode::NOT_FOUND);
    }

    fn failing_app() -> Element {
        rsx! {
            ErrorBoundary {
                handle_error: render_error,
                FailingComponent {}
            }
        }
    }

    #[component]
    fn FailingComponent() -> Element {
        Err(std::io::Error::other("unexpected SSR failure"))?;
        rsx! {}
    }

    #[tokio::test]
    async fn unexpected_ssr_failure_returns_500() {
        assert_eq!(
            render_status(failing_app, "/").await,
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_route_has_useful_metadata() {
        let routes = [
            Route::Home {},
            Route::Blog {},
            Route::Projects {},
            Route::AboutMe {},
            Route::Guestbook {},
            Route::NotFound {
                route: vec!["missing".to_string()],
            },
        ];

        for route in routes {
            let metadata = route.metadata();
            assert!(metadata.title.contains("George Andreev"));
            assert!(metadata.description.len() >= 60);
        }
    }
}

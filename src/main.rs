#![allow(non_snake_case)]
use dioxus::fullstack::FullstackContext;
use dioxus::prelude::*;
use std::{borrow::Cow, str::FromStr};
use tracing::Level;
mod auth;
#[cfg(feature = "server")]
mod backend;
mod blog;
mod components;
#[cfg(feature = "server")]
mod hide;
mod pages;
pub mod shared;
use components::layout::Layout;
use pages::{AboutMe, Blog, BlogPost, Guestbook, Home, NotFound, Projects};
static STYLES: Asset = asset!("/assets/styles");
fn main() -> anyhow::Result<()> {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let log_level = Level::from_str(&log_level).unwrap_or(Level::INFO);
    #[cfg(not(feature = "server"))]
    dioxus_logger::init(log_level).expect("failed to init logger");
    #[cfg(feature = "server")]
    let _sentry = backend::observability::init(log_level);
    #[cfg(not(feature = "server"))]
    LaunchBuilder::new()
        .with_cfg(web! {
            dioxus::web::Config::new().hydrate(true)
        })
        .launch(App);
    #[cfg(feature = "server")]
    {
        tracing::info!("Starting server");
        // Dioxus 0.7.10 incremental cache hits currently lose custom route statuses.
        let config = ServeConfig::new().enable_out_of_order_streaming();
        tokio::runtime::Runtime::new()?.block_on(backend::server::serve(config, App))?;
    }
    Ok(())
}
#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/blog")]
    Blog {},
    #[route("/blog/:slug")]
    BlogPost { slug: String },
    #[route("/projects")]
    Projects {},
    #[route("/about")]
    AboutMe {},
    #[route("/guestbook")]
    Guestbook {},
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

const SITE_ORIGIN: &str = "https://gabioinf.dev";

#[derive(Clone, Debug, Eq, PartialEq)]
struct RouteMetadata {
    title: Cow<'static, str>,
    description: Cow<'static, str>,
    canonical_path: Option<Cow<'static, str>>,
    page_type: &'static str,
    noindex: bool,
}

impl RouteMetadata {
    fn page(title: &'static str, description: &'static str, path: &'static str) -> Self {
        Self {
            title: Cow::Borrowed(title),
            description: Cow::Borrowed(description),
            canonical_path: Some(Cow::Borrowed(path)),
            page_type: "website",
            noindex: false,
        }
    }

    fn not_found() -> Self {
        Self {
            title: Cow::Borrowed("Page not found | George Andreev"),
            description: Cow::Borrowed(
                "The requested page could not be found on George Andreev's website.",
            ),
            canonical_path: None,
            page_type: "website",
            noindex: true,
        }
    }

    fn canonical_url(&self) -> Option<String> {
        self.canonical_path
            .as_deref()
            .map(|path| format!("{SITE_ORIGIN}{path}"))
    }
}

impl Route {
    fn metadata(&self) -> RouteMetadata {
        match self {
            Self::Home {} => RouteMetadata::page(
                "George Andreev | Bioinformatician and Developer",
                "George Andreev's personal website about bioinformatics, software development, projects, and experiments.",
                "/",
            ),
            Self::Blog {} => RouteMetadata::page(
                "Blog | George Andreev",
                "Notes and articles from George Andreev about bioinformatics, software development, and other experiments.",
                "/blog",
            ),
            Self::BlogPost { slug } => {
                crate::blog::find_post(slug).map_or_else(RouteMetadata::not_found, |post| {
                    RouteMetadata {
                        title: Cow::Owned(format!("{} | George Andreev", post.title)),
                        description: Cow::Borrowed(post.description),
                        canonical_path: Some(Cow::Owned(format!("/blog/{}", post.slug))),
                        page_type: "article",
                        noindex: false,
                    }
                })
            }
            Self::Projects {} => RouteMetadata::page(
                "Projects | George Andreev",
                "Selected software projects, publications, and professional milestones from George Andreev.",
                "/projects",
            ),
            Self::AboutMe {} => RouteMetadata::page(
                "About | George Andreev",
                "Learn about George Andreev, a bioinformatician and software developer working on machine learning for biology.",
                "/about",
            ),
            Self::Guestbook {} => RouteMetadata::page(
                "Guestbook | George Andreev",
                "Read messages from visitors and sign George Andreev's guestbook with GitHub.",
                "/guestbook",
            ),
            Self::NotFound { .. } => RouteMetadata::not_found(),
        }
    }
}

fn DocumentMetadata() -> Element {
    let route: Route = router().current();
    let metadata = route.metadata();
    let canonical_url = metadata.canonical_url();
    let article = match &route {
        Route::BlogPost { slug } => crate::blog::find_post(slug),
        _ => None,
    };
    let robots = if metadata.noindex {
        "noindex, nofollow"
    } else {
        "index, follow"
    };

    rsx! {
        document::Title { "{metadata.title}" }
        document::Meta { name: "description", content: metadata.description.to_string() }
        if let Some(canonical_url) = canonical_url {
            document::Link { rel: "canonical", href: canonical_url.clone() }
            document::Meta { property: "og:url", content: canonical_url }
        }
        document::Link {
            rel: "alternate",
            r#type: "application/atom+xml",
            title: "George Andreev's blog",
            href: "/feed.xml",
        }
        document::Meta { property: "og:title", content: metadata.title.to_string() }
        document::Meta {
            property: "og:description",
            content: metadata.description.to_string(),
        }
        document::Meta { property: "og:type", content: metadata.page_type }
        document::Meta { property: "og:image", content: "{SITE_ORIGIN}/assets/og-img.png" }
        document::Meta { property: "og:image:alt", content: "George Andreev's personal website" }
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "robots", content: robots }
        if let Some(post) = article {
            document::Meta {
                property: "article:published_time",
                content: format!("{}T00:00:00Z", post.published),
            }
            if let Some(updated) = post.updated {
                document::Meta {
                    property: "article:modified_time",
                    content: format!("{updated}T00:00:00Z"),
                }
            }
            document::Meta { property: "article:author", content: "George Andreev" }
            for tag in post.tags {
                document::Meta { property: "article:tag", content: (*tag).to_string() }
            }
        }
    }
}

fn App() -> Element {
    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Meta { charset: "UTF-8" }
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Stylesheet { href: "{STYLES}/fonts.css" }
        document::Stylesheet { href: asset!("assets/tailwind.css") }
        ErrorBoundary {
            handle_error: render_error,
            AppRouter {}
        }
    }
}

#[component]
fn AppRouter() -> Element {
    rsx! { Router::<Route> {} }
}

fn render_error(errors: ErrorContext) -> Element {
    let error = errors
        .error()
        .expect("error boundary must contain an error");
    FullstackContext::commit_error_status(error.clone());

    rsx! {
        article { class: "flex flex-col gap-5",
            span { class: "label-mono", "// error" }
            h1 { class: "heading-casual m-0 text-[34px] leading-[1.15]", "Something went wrong." }
            p { class: "prose-font m-0 text-lg text-muted", "An unexpected error occurred." }
            code { class: "overflow-x-auto rounded-md border border-card bg-code p-4 text-sm text-muted", "{error}" }
            p { class: "prose-font m-0 text-lg text-muted",
                "If you think this is a mistake, please "
                a {
                    href: "https://github.com/LilDojd/gabioinf/issues/new",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "link-dashed",
                    "open an issue on GitHub"
                }
                "."
            }
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod ssr_tests {
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

    #[tokio::test]
    async fn public_blog_routes_do_not_require_authentication() {
        assert_eq!(render_status(App, "/blog").await, StatusCode::OK);
        assert_eq!(
            render_status(App, "/blog/missing").await,
            StatusCode::NOT_FOUND
        );
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
            Route::BlogPost {
                slug: "missing".to_string(),
            },
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

        let missing = Route::BlogPost {
            slug: "missing".to_string(),
        }
        .metadata();
        assert!(missing.noindex);
        assert!(missing.canonical_url().is_none());
    }
}

use dioxus::prelude::*;
mod components;
mod pages;
use components::layout::NavFooter;
use pages::{AboutMe, Home};
#[cfg(feature = "server")]
mod backend;
const _TAILWIND_URL: &str = manganis::mg!(file("assets/tailwind.css"));
const _STYLE: &str = manganis::mg!(file("assets/main.css"));
fn main() {
    #[cfg(feature = "web")]
    tracing_wasm::set_as_global_default();
    #[cfg(feature = "server")]
    tracing_subscriber::fmt::init();

    LaunchBuilder::fullstack()
        .with_cfg(server_only!(ServeConfig::builder().incremental(
            IncrementalRendererConfig::default()
                .invalidate_after(std::time::Duration::from_secs(120)),
        )))
        .launch(App);
}
#[derive(Clone, Debug)]
pub enum AuthState {
    LoggedIn,
    LoggedOut,
}
#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[layout(NavFooter)]
    #[route("/")]
    Home {},
    #[route("/about")]
    AboutMe {},
}
fn App() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 flex justify-center items-center min-h-screen",
            Router::<Route> {}
        }
    }
}

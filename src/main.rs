use dioxus::prelude::*;
use dioxus_fullstack::prelude::*;
mod components;
mod pages;
use pages::Home;

use components::layout::NavFooter;

#[cfg(feature = "server")]
mod backend;

#[cfg(feature = "server")]
mod server;

#[cfg(feature = "web")]
const _TAILWIND_URL: &str = manganis::mg!(file("assets/tailwind.css"));

fn main() {
    #[cfg(feature = "web")]
    {
        tracing_wasm::set_as_global_default();
        dioxus_web::launch::launch_cfg(app, dioxus_web::Config::new().hydrate(true));
    }
    #[cfg(feature = "server")]
    {
        tracing_subscriber::fmt::init();
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move { server::serve().await });
    }
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
    #[redirect("/:..segments", |segments:Vec<String>|Route::Home{})]
    Home {},
}

fn app() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

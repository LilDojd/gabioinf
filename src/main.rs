use dioxus::prelude::*;

#[cfg(feature = "server")]
mod backend;

#[cfg(feature = "server")]
mod server;

fn main() {
    #[cfg(feature = "web")]
    {
        tracing_wasm::set_as_global_default();
        launch(app)
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
    #[route("/")]
    #[redirect("/:..segments", |segments:Vec<String>|Route::Home{})]
    Home {},
}

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            "Hello, World!"
        }
    }
}

fn app() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

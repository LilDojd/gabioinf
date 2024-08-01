use dioxus::prelude::*;
mod components;
mod pages;
use components::layout::NavFooter;
use reqwest::{
    header::{CONTENT_TYPE, SET_COOKIE, X_CONTENT_TYPE_OPTIONS},
    Client,
};
const _TAILWIND_URL: &str = manganis::mg!(file("assets/tailwind.css"));
#[derive(Clone, Debug)]
pub enum AuthState {
    LoggedIn,
    LoggedOut,
}
pub async fn check_auth_status() -> AuthState {
    let client = Client::new();
    let resp = client
        .get("http://localhost:8000/v1/auth/status")
        .send()
        .await;
    match resp {
        Ok(resp) => {
            if resp.status().is_success() {
                AuthState::LoggedIn
            } else {
                AuthState::LoggedOut
            }
        }
        Err(_) => AuthState::LoggedOut,
    }
}
#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[layout(NavFooter)]
    #[route("/")]
    #[redirect("/:..segments", |segments:Vec<String>|Route::HomePage{})]
    HomePage {},
    #[end_layout]
    #[route("/login")]
    LoginPage {},
    #[route("/protected")]
    ProtectedPage {},
}
fn app() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
fn main() {
    tracing_wasm::set_as_global_default();
    launch(app);
}
#[component]
fn LoginPage() -> Element {
    rsx! {
        div {
            Link { to: "http://localhost:8000/v1/login", "Login" }
        }
    }
}
#[component]
fn HomePage() -> Element {
    let auth_state = use_resource(check_auth_status);
    rsx! {
        div {
            h2 { "Welcome to GABioInf Guestbook" }
            match * auth_state.read() {
            Some(AuthState::LoggedIn) => { rsx! { Link { to : "/protected",
            "Go to protected page" } } } Some(AuthState::LoggedOut) => { rsx! { Link { to :
            "/login", "Login" } } } None => { rsx! { "Loading..." } } }
        }
    }
}
#[component]
fn ProtectedPage() -> Element {
    let auth_state = use_resource(check_auth_status);
    rsx! {
        div {
            h2 { "Protected Page" }
            match * auth_state.read() {
            Some(AuthState::LoggedIn) => { rsx! { "You are logged in!" } }
            Some(AuthState::LoggedOut) => { rsx! { "You are not logged in!" } } None => {
            rsx! { "Loading..." } } }
        }
    }
}

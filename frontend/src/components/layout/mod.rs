mod footer;
use footer::Footer;
mod navbar;
use crate::Route;
use dioxus::prelude::*;
use dioxus_router::components::Outlet;
use navbar::Navbar;
#[component]
pub fn NavFooter() -> Element {
    rsx! {
        div {
            Navbar {}
            Outlet::<Route> {}
        }
        Footer {}
    }
}

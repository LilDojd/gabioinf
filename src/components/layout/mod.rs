use crate::Route;
use dioxus::prelude::*;
use dioxus_router::components::Outlet;
mod footer;
mod navbar;
use footer::Footer;
use navbar::Navbar;
#[component]
pub fn NavFooter() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 md:px-6 py-6 md:py-10 min-h-screen flex flex-col",
            Navbar {}
            div { class: "flex flex-grow justify-center max-w-4xl mx-auto w-full pt-4 md:pt-8 pb-20",
                Outlet::<Route> {}
            }
            Footer {}
        }
    }
}

use crate::Route;
use dioxus::prelude::*;
#[component]
pub fn Navbar() -> Element {
    rsx! {
        aside { class: "mb-8 mt-2 md:mt-4 tracking-tight",
            nav { class: "flex flex-row justify-center px-0 pb-0 overflow-visible  md:space-x-8",
                NavItem { to: Route::Home {}, label: "home" }
                NavItem { to: Route::Blog {}, label: "blog" }
                NavItem { to: Route::Projects {}, label: "projects" }
                NavItem { to: Route::AboutMe {}, label: "about me" }
                NavItem { to: Route::Guestbook {}, label: "guestbook" }
            }
        }
    }
}
#[component]
fn NavItem(to: Route, label: &'static str) -> Element {
    let route: Route = use_route();
    let is_active = route.to_string() == to.to_string();
    rsx! {
        Link {
            to,
            class: format!(
                "nav-link flex items-center py-1 px-2 text-base md:text-lg {}",
                if is_active { "active" } else { "" },
            ),
            "{label}"
        }
    }
}

use crate::Route;
use dioxus::prelude::*;
#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav { class: "bg-nasty-black",
            div { class: "container mx-auto flex justify-center items-center",
                ul { class: "flex space-x-8",
                    li {
                        Link { to: Route::Home {}, class: "text-white hover:text-gray-300 text-lg", "home" }
                    }
                    li {
                        Link { to: Route::Home {}, class: "text-white hover:text-gray-300 text-lg", "blog" }
                    }
                    li {
                        Link { to: Route::AboutMe {}, class: "text-white hover:text-gray-300 text-lg", "about me" }
                    }
                    li {
                        Link { to: Route::Home {}, class: "text-white hover:text-gray-300 text-lg", "guestbook" }
                    }
                }
            }
        }
    }
}

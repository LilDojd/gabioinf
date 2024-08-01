use chrono::Datelike;
use dioxus::prelude::*;
#[component]
pub fn Footer() -> Element {
    let current_year = chrono::Utc::now().year();
    rsx! {
        footer { class: "bg-nasty-black text-white",
            div { class: "container mx-auto flex justify-between items-center",
                a {
                    href: "https://www.linkedin.com/in/georgiy-andreev/",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "text-white hover:text-gray-300",
                    img {
                        src: "/linkedin.svg",
                        alt: "LinkedIn link",
                        class: "w-6 h-6"
                    }
                }
                p { class: "text-sm", "{current_year}" }
            }
        }
    }
}

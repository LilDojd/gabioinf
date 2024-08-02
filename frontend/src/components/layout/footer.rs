use chrono::Datelike;
use dioxus::prelude::*;
#[component]
pub fn Footer() -> Element {
    let current_year = chrono::Utc::now().year();
    rsx! {
        footer { class: "bg-nasty-black text-white fixed bottom-0 w-full dark:bg-grey-950 z-1000",
            div { class: "container mx-auto flex justify-between items-center",
                a {
                    href: "https://www.linkedin.com/in/georgiy-andreev/",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "text-white hover:text-gray-300",
                    img {
                        src: "/linkedin.svg",
                        alt: "LinkedIn link",
                        class: "w-4 h-4"
                    }
                }
                p { class: "text-sm", "{current_year}" }
            }
        }
    }
}

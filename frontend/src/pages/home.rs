use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "relative min-h-screen bg-nasty-black text-white",
            div { class: "absolute top-0 left-0 w-1/2 h-1/2",
                video {
                    class: "absolute top-0 left-0 object-cover",
                    playsinline: true,
                    autoplay: true,
                    muted: true,
                    r#loop: "true",
                    src: "/alien_white.webm"
                }
            }
            main {
                class: "relative z-10 flex flex-col items-center justify-center min-h-screen text-center px-4",
                h1 { class: "text-4xl font-bold mb-2", "George" }
                p { class: "text-xl mb-8", "Software Engineer at GABioInf" }
                p { class: "max-w-2xl mb-8",
                    "Building polished software experiences with magical, unique and delightful details, for the web. I aim to create beautiful and functional software that is both intuitive and enjoyable for users."
                }
                p { class: "mb-8 max-w-2xl",
                    "I have a passion for learning, and I am constantly seeking to improve my skills mostly through "
                    span { class: "underline", "reading" }
                    " and "
                    span { class: "underline", "writing" }
                    ". I'm interested in Rust and at the same time, I'm also experimenting with other technologies."
                }
                div { class: "flex space-x-4",
                    Link { to: Route::Home {}, class: "underline", "follow me on x" }
                    Link { to: Route::Home {}, class: "underline", "let's collaborate on github" }
                    Link { to: Route::Home {}, class: "underline", "love to talk?" }
                }
            }
        }
    }
}

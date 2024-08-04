use crate::Route;
use dioxus::prelude::*;
#[component]
pub fn Home() -> Element {
    let mut video_loaded = use_signal(|| "visible".to_string());
    rsx! {
        div { class: "min-h-screen bg-nasty-black text-white flex flex-col justify-center items-center p-4",
            div { class: "container mx-auto max-w-5xl",
                div { class: "flex flex-col lg:flex-row items-center justify-center gap-8",
                    div { class: "lg:w-1/2 text-left",
                        h1 { class: "text-4xl font-bold mb-4", "Hey, I'm George" }
                        p { class: "text-lg mb-4",
                            "I'm a bioinformatician gone rogue to become a developer."
                        }
                        p { class: "text-lg mb-6",
                            "I welcome you to read my random rambles, checkout my blogs or learn more about me and sign my guestbook"
                        }
                        div { class: "flex flex-wrap gap-4",
                            Link { to: Route::Home {}, class: "underline", "follow me on linkedin" }
                            Link { to: Route::Home {}, class: "underline", "i have some stuff on github" }
                            Link { to: Route::Home {}, class: "underline", "fancy a chat?" }
                        }
                    }
                    div { class: "lg:w-1/2 w-full max-w-md relative",
                        video {
                            class: "w-full h-full object-cover",
                            playsinline: true,
                            autoplay: true,
                            muted: true,
                            r#loop: "true",
                            src: "/alien_white.webm"
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
#[component]
pub fn UnderConstruction() -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center text-center select-none",
            h1 { class: "text-6xl font-bold text-alien-green mb-4", "503" }
            p { class: "text-2xl text-stone-300 mb-8",
                "Oops! This page vanished from our universe. Come back later!"
            }
        }
    }
}

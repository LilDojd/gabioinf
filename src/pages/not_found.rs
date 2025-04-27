use dioxus::prelude::*;
#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    let galaxy = asset!(
        "/assets/galaxy.png",
        ImageAssetOptions::new().with_avif().with_preload(true)
    );
    rsx! {
        div { class: "flex flex-col items-center justify-center text-center w-full select-none",
            div { class: "w-full h-1/2 relative",
                div {
                    class: "absolute inset-0 bg-no-repeat bg-cover bg-center blur-[2px] [mask-image:linear-gradient(45deg,white,rgba(255,255,255,0))]",
                    style: "background-image: url({galaxy})",
                }
                div { class: "absolute inset-0 flex items-center justify-center",
                    h1 { class: "text-8xl font-bold text-alien-green", "404" }
                }
            }
            p { class: "text-2xl text-stone-300 mb-8", "you got here.. but at what cost?" }
        }
    }
}

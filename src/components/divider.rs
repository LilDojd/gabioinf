use dioxus::prelude::*;
#[component]
pub fn Hr(comment: Option<String>) -> Element {
    match comment {
        Some(inner_text) => {
            rsx! {
                div { class: "not-prose inline-flex items-center justify-center w-full mb-4 mt-0",
                    hr { class: "h-px w-full bg-jet border-0" }
                    span { class: "text-stone-400 px-3 absolute -translate-x-1/2 left-1/2 bg-nasty-black text-sm",
                        "{inner_text}"
                    }
                }
            }
        }
        None => {
            rsx! {
                div { class: "not-prose mb-4 mt-0",
                    hr { class: "h-px bg-jet border-0" }
                }
            }
        }
    }
}

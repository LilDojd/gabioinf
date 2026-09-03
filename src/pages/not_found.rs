use crate::Route;
use dioxus::fullstack::FullstackContext;
use dioxus::prelude::*;

#[component]
pub fn NotFound(route: Vec<String>) -> Element {
    let _ = route;
    FullstackContext::commit_http_status(StatusCode::NOT_FOUND, None);

    rsx! {
        section { class: "flex flex-col gap-5 pt-6",
            span { class: "label-mono", "// 404" }
            h1 { class: "heading-casual m-0 text-pretty text-[34px] leading-[1.15]", "This page vanished from our universe." }
            p { class: "m-0 text-muted",
                "Come back later, or head "
                Link { to: Route::Home {}, class: "link-dashed", "home" }
                "."
            }
        }
    }
}

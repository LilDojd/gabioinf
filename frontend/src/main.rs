use dioxus::prelude::*;

fn main() {
    launch(app)
}

fn app() -> Element {
    rsx! {
        div {
            h1 { "Welcome to My Personal Website" }
            p { "This is a simple Dioxus app." }
        }
    }
}

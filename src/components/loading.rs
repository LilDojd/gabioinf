use dioxus::prelude::*;
#[component]
pub fn Loading() -> Element {
    rsx! {
        Spinner {}
    }
}
fn Spinner() -> Element {
    rsx! {
        div { class: "relative rounded-xl overflow-auto p-6",
            div { class: "flex items-center justify-center",
                button {
                    r#type: "button",
                    disabled: "",
                    class: "inline-flex items-center px-4 py-2 font-semibold leading-6 text-sm shadow rounded-md text-stone-300 bg-onyx cursor-not-allowed",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        class: "animate-spin -ml-1 mr-3 h-5 w-5 text-alien-green",
                        circle {
                            r: "10",
                            cx: "12",
                            stroke: "currentColor",
                            cy: "12",
                            stroke_width: "4",
                            class: "opacity-25",
                        }
                        path {
                            fill: "currentColor",
                            d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                            class: "opacity-75",
                        }
                    }
                    "Loading..."
                }
            }
        }
    }
}

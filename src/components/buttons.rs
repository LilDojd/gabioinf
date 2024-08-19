use dioxus::prelude::*;

#[derive(Props, Clone, Debug, PartialEq)]
pub struct CloseButtonProps {
    pub layout: String,
    pub onclick: EventHandler<MouseEvent>,
}

#[component]
pub fn CloseButton(props: CloseButtonProps) -> Element {
    rsx! {
        button {
            class: "{props.layout} text-stone-400 hover:text-coral flex items-center justify-center rounded-lg border border-stone-400 hover:border-coral transition-colors duration-200 leading-none",
            onclick: move |evt| props.onclick.call(evt),
            span { class: "relative", style: "top: -1px;", "×" }
        }
    }
}

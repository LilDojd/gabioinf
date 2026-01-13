use dioxus::prelude::*;
#[derive(Props, Clone, PartialEq, Debug)]
pub struct StyledButtonProps {
    text: String,
    onclick: EventHandler<MouseEvent>,
    #[props(default)]
    variant: ButtonVariant,
    #[props(default)]
    class: String,
    #[props(default = "button".to_string())]
    r#type: String,
    #[props(default)]
    icon: IconVariant,
}
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Default)]
pub enum IconVariant {
    #[default]
    None,
    Asset(String),
    Rsx(Element),
}
#[derive(Clone, Debug, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
}
#[component]
pub fn StyledButton(props: StyledButtonProps) -> Element {
    let base_classes = "px-4 py-2 rounded-lg font-semibold transition duration-200 ease-in-out flex items-center border";
    let (bg_color, text_color, hover_color, border_color) = match props.variant {
        ButtonVariant::Primary => ("bg-jet", "text-stone-100", "hover:bg-onyx", "border-onyx"),
        ButtonVariant::Secondary => (
            "bg-transparent",
            "text-stone-100",
            "hover:bg-onyx",
            "border-transparent",
        ),
    };
    rsx! {
        button {
            class: "{base_classes} {bg_color} {text_color} {hover_color} {border_color} {props.class}",
            r#type: "{props.r#type}",
            onclick: move |evt| props.onclick.call(evt),
            {
                match props.icon {
                    IconVariant::Asset(asset) => rsx! {
                        img { src: "{asset}", alt: "Button icon", class: "w-5 h-5 mr-2" }
                    },
                    IconVariant::Rsx(icon) => rsx! {
                        div { class: "w-5 h-5 mr-2", {icon} }
                    },
                    IconVariant::None => rsx! {},
                }
            }
            "{props.text}"
        }
    }
}

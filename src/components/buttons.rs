use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    pub children: Element,
    pub onclick: EventHandler<MouseEvent>,
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub disabled: bool,
    #[props(default = "button".to_string())]
    pub r#type: String,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            class: match props.variant { ButtonVariant::Primary => "btn-primary", ButtonVariant::Secondary => "btn-secondary" },
            r#type: props.r#type,
            disabled: props.disabled,
            onclick: move |event| props.onclick.call(event),
            {props.children}
        }
    }
}

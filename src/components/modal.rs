use dioxus::prelude::*;

#[component]
pub fn Modal(
    on_close: EventHandler<()>,
    label: &'static str,
    #[props(default)] top: bool,
    panel_class: &'static str,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex justify-center bg-[rgba(10,11,13,.6)] p-5 backdrop-blur-sm",
            class: if top { "items-start pt-[18vh]" } else { "items-center" },
            onclick: move |_| on_close.call(()),
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    on_close.call(());
                }
            },
            div {
                role: "dialog",
                aria_modal: "true",
                aria_label: label,
                class: "rounded-lg border border-border-strong bg-surface shadow-[0_20px_60px_rgba(0,0,0,.5)] {panel_class}",
                onclick: move |event| event.stop_propagation(),
                {children}
            }
        }
    }
}

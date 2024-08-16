use dioxus::prelude::*;
use serde::Deserialize;
#[derive(Props, Clone, Debug, PartialEq)]
pub struct CardProps {
    card_type: CardType,
    #[props(default)]
    class: String,
}
#[derive(Clone, Debug, PartialEq)]
pub enum CardType {
    Project(Project),
    Signature(GuestbookEntry),
}
#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct GuestbookEntry {
    pub message: String,
    pub signature: String,
    pub date: String,
    pub username: String,
}
#[component]
pub fn Card(props: CardProps) -> Element {
    let base_class = "bg-jet rounded-lg shadow-lg p-6 border border-onyx ease-in-out transition-colors duration-200";
    match &props.card_type {
        CardType::Project(project) => {
            rsx! {
                div { class: "{base_class} {props.class} hover:border-alien-green flex flex-col h-full",
                    div { class: "flex-grow",
                        h3 { class: "text-xl font-semibold mb-2 text-stone-100",
                            {
                                if let Some(url) = &project.url {
                                    rsx! {
                                        Link {
                                            to: "{url}",
                                            new_tab: true,
                                            class: "flex items-center hover:text-alien-green",
                                            "{project.name}"
                                        }
                                    }
                                } else {
                                    rsx! {
                                    "{project.name}"
                                    }
                                }
                            }
                        }
                        p { class: "text-stone-400 mb-4", "{project.description}" }
                    }
                    {
                        if let Some(url) = &project.url {
                            rsx! {
                                div { class: "mt-auto pt-4",
                                    ProjectLink { url: url.clone() }
                                }
                            }
                        } else {
                            rsx! {  }
                        }
                    }
                }
            }
        }
        CardType::Signature(entry) => {
            rsx! {
                div { class: "{base_class} {props.class} flex flex-col justify-between space-y-3 h-full",
                    p { class: "text-stone-100 leading-6", "{entry.message}" }
                    div { class: "mt-auto flex items-center justify-between",
                        div { class: "flex flex-col justify-end h-full text-sm text-stone-400",
                            p {
                                "by "
                                span { class: "font-bold", "{entry.username}" }
                            }
                            p { "{entry.date}" }
                        }
                        img {
                            class: "w-[150px] max-h-[150px] -mb-4 -mr-4",
                            src: "data:image/png;base64,{entry.signature}",
                            alt: "Signature",
                        }
                    }
                }
            }
        }
    }
}
#[component]
pub fn ProjectLink(url: String) -> Element {
    rsx! {
        Link {
            to: "{url}",
            new_tab: true,
            class: "inline-flex items-center text-stone-300 hover:text-alien-green transition-colors duration-200",
            "View Project"
            svg {
                class: "w-4 h-4 ml-2",
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                view_box: "0 0 24 24",
                stroke: "currentColor",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M14 5l7 7m0 0l-7 7m7-7H3",
                }
            }
        }
    }
}
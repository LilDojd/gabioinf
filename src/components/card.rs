use crate::shared::models::GuestbookEntry;
use dioxus::prelude::*;
use serde::Deserialize;
use time::UtcOffset;
#[derive(Props, Clone, Debug, PartialEq)]
pub struct CardProps {
    card_type: CardType,
    #[props(default)]
    class: String,
}
#[derive(Clone, Debug, PartialEq)]
pub enum CardType {
    Project(Project),
    Signature { entry: GuestbookEntry, close_button: Element },
    Skeleton,
}
#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
}
#[component]
pub fn Card(props: CardProps) -> Element {
    let base_class = "bg-jet rounded-lg shadow-lg p-6 border border-onyx ease-in-out transition-colors duration-200";
    match props.card_type {
        CardType::Project(project) => {
            rsx! {
                div { class: "{base_class} {props.class} hover:border-alien-green flex flex-col h-full",
                    div { class: "flex-grow",
                        h3 { class: "text-xl font-semibold mb-2 text-stone-100",
                            {
                                if let Some(url) = &project.url {
                                    rsx! {
                                        a {
                                            href: "{url}",
                                            rel: "noopener noreferrer",
                                            target: "_blank",
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
                            rsx! {}
                        }
                    }
                }
            }
        }
        CardType::Signature { entry, close_button } => {
            let sigb64 = entry.signature.clone().unwrap_or_default();
            let date = entry.created_at;
            let offset = UtcOffset::local_offset_at(date).unwrap_or(date.offset());
            let date = date
                .to_offset(offset)
                .format(
                    time::macros::format_description!(
                        "[day] [month repr:short], [year], [hour repr:24]:[minute]"
                    ),
                )
                .unwrap()
                .to_string();
            rsx! {
                div { class: "{base_class} {props.class} flex flex-col justify-between h-full relative p-6",
                    {close_button}
                    div { class: "flex=grow",
                        p { class: "text-stone-100 leading-6 mt-0", "{entry.message}" }
                    }
                    div { class: "mt-3 flex items-center justify-between",
                        div { class: "flex flex-col justify-end h-full text-sm text-stone-400",
                            p {
                                "by "
                                span { class: "font-bold", "{entry.author_username}" }
                            }
                            p { "{date}" }
                        }
                        img {
                            class: "w-[200px] max-h-[175px] -mb-4 -mr-4",
                            src: "data:image/png;base64,{sigb64}",
                            alt: "Signature",
                        }
                    }
                }
            }
        }
        CardType::Skeleton => {
            rsx! {
                div { class: "{base_class} {props.class} p-6",
                    div { class: "h-4 bg-onyx rounded w-3/4 mb-12 animate-pulse" }
                    div { class: "h-4 bg-onyx rounded w-1/4 mb-2 animate-pulse" }
                    div { class: "h-4 bg-onyx rounded w-1/2 animate-pulse" }
                }
            }
        }
    }
}
#[component]
pub fn ProjectLink(url: String) -> Element {
    rsx! {
        a {
            href: "{url}",
            rel: "noopener noreferrer",
            target: "_blank",
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

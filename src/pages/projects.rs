use crate::components::{Card, CardType, Project};
use dioxus::prelude::*;
const PROJECTS_SOURCE: &str = include_str!("../../data/projects.yml");
#[component]
pub fn Projects() -> Element {
    let projects: Vec<Project> =
        serde_yaml::from_str(PROJECTS_SOURCE).expect("Unable to parse YAML");
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            article { class: "prose prose-invert prose-stone prose-h2:mb-0 lg:prose-lg mb-8",
                h1 { "projects" }
                p {
                    "A collection of public stuff I have been working on over the years, including milestones, publications and coding projects."
                }
            }
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                {projects.iter().map(|project| rsx! {
                    Card { card_type: CardType::Project(project.clone()) }
                })}
            }
        }
    }
}

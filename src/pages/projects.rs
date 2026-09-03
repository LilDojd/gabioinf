use dioxus::prelude::*;
use std::fmt;

#[derive(PartialEq)]
struct Project {
    name: &'static str,
    description: &'static str,
    url: Option<&'static str>,
    kind: ProjectKind,
}

#[derive(Clone, Copy, PartialEq)]
enum ProjectKind {
    Code,
    Work,
    Milestone,
    Publication,
    Research,
}

impl fmt::Display for ProjectKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Code => "code",
            Self::Work => "work",
            Self::Milestone => "milestone",
            Self::Publication => "publication",
            Self::Research => "research",
        })
    }
}

static PROJECTS: &[Project] = &[
    Project {
        name: "gabioinf",
        description: "This website.",
        url: Some("https://github.com/LilDojd/gabioinf"),
        kind: ProjectKind::Code,
    },
    Project {
        name: "Alchemistry",
        description: "SOTA Alchemical free energy calculations for drug discovery",
        url: Some("https://insilico.com/chemistry42#rec745522589"),
        kind: ProjectKind::Work,
    },
    Project {
        name: "ISM001-055",
        description: "AI-discovered drug now in Phase 2 clinical trials for IPF",
        url: Some(
            "https://www.forbes.com/sites/calumchace/2022/02/25/first-wholly-ai-developed-drug-enters-phase-1-trials/",
        ),
        kind: ProjectKind::Milestone,
    },
    Project {
        name: "Generative Hit-Opt with Alchemistry",
        description: "A diffusion-based generative model for small molecule design",
        url: None,
        kind: ProjectKind::Work,
    },
    Project {
        name: "Colabind",
        description: "A Cloud-based approach for prediction of binding sites",
        url: Some("https://pubs.acs.org/doi/10.1021/acs.jpcb.3c07853"),
        kind: ProjectKind::Publication,
    },
    Project {
        name: "YoungFace",
        description: "Winners of LongHack 2022",
        url: None,
        kind: ProjectKind::Milestone,
    },
    Project {
        name: "NN-Enhanced ABFE",
        description: "ML/MM for binding free energy",
        url: None,
        kind: ProjectKind::Research,
    },
    Project {
        name: "dioxus-spline",
        description: "Spline scenes in Dioxus!",
        url: Some("https://crates.io/crates/dioxus-spline"),
        kind: ProjectKind::Code,
    },
];

#[component]
pub fn Projects() -> Element {
    rsx! {
        section { class: "flex flex-col gap-9",
            header { class: "flex flex-col gap-3",
                span { class: "label-mono", "// projects" }
                h1 { class: "heading-casual m-0 text-[30px] leading-[1.2]", "public stuff" }
                p { class: "prose-font m-0 text-pretty text-lg text-muted",
                    "A collection of public stuff I have been working on over the years, including milestones, publications and coding projects."
                }
            }
            div { class: "grid grid-cols-1 gap-2.5 min-[760px]:grid-cols-2",
                for project in PROJECTS {
                    if let Some(url) = project.url {
                        a {
                            key: "{project.name}",
                            href: url,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "card flex flex-col gap-2 p-4 text-text no-underline hover:border-accent",
                            ProjectContent { project }
                        }
                    } else {
                        div { key: "{project.name}", class: "card flex flex-col gap-2 p-4",
                            ProjectContent { project }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProjectContent(project: &'static Project) -> Element {
    rsx! {
        span { class: "label-mono", "{project.kind}" }
        span { class: "text-base leading-[1.3] [font-variation-settings:'CASL'_.6,'wght'_600]", "{project.name}" }
        span { class: "text-pretty text-sm leading-[1.5] text-muted", "{project.description}" }
    }
}

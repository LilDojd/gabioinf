use crate::Route;
use dioxus::prelude::*;
#[component]
pub fn AboutMe() -> Element {
    rsx! {
        div { class: "min-h-screen bg-nasty-black text-white p-8",
            div { class: "max-w-3xl mx-auto",
                h1 { class: "text-4xl font-bold mb-8", "About Me" }
                section { class: "mb-12",
                    h2 { class: "text-2xl font-semibold mb-4", "Hello, I'm George" }
                    p { class: "mb-4",
                        "I'm a bioinformatician turned developer, blending the worlds of biology and technology to create innovative solutions."
                    }
                    p { class: "mb-4",
                        "My journey from analyzing genomes to crafting code has been an exciting adventure, driven by my passion for both sciences and programming."
                    }
                }
                section { class: "mb-12",
                    h2 { class: "text-2xl font-semibold mb-4", "My Background" }
                    p { class: "mb-4",
                        "With a strong foundation in bioinformatics, I've spent years working with complex biological data, developing algorithms, and creating tools to analyze genetic information."
                    }
                    p { class: "mb-4",
                        "This unique background has given me a different perspective on problem-solving and data analysis, which I now apply to various domains in software development."
                    }
                }
                section { class: "mb-12",
                    h2 { class: "text-2xl font-semibold mb-4", "Skills & Expertise" }
                    ul { class: "list-disc list-inside space-y-2",
                        li { "Programming Languages: Python, R, Rust, JavaScript" }
                        li { "Web Development: HTML, CSS, React, Dioxus" }
                        li { "Data Analysis & Visualization" }
                        li { "Machine Learning & Bioinformatics Algorithms" }
                        li { "Database Management: SQL, MongoDB" }
                        li { "Version Control: Git" }
                    }
                }
                section { class: "mb-12",
                    h2 { class: "text-2xl font-semibold mb-4", "Current Focus" }
                    p { class: "mb-4",
                        "I'm currently exploring the intersection of web development and data science, building interactive applications that make complex data accessible and understandable."
                    }
                    p { class: "mb-4",
                        "My goal is to bridge the gap between cutting-edge research and practical, user-friendly software solutions."
                    }
                }
                section { class: "mb-12",
                    h2 { class: "text-2xl font-semibold mb-4", "Beyond Coding" }
                    p { class: "mb-4", "When I'm not immersed in code, you can find me:" }
                    ul { class: "list-disc list-inside space-y-2",
                        li { "Reading sci-fi novels and scientific papers" }
                        li { "Experimenting with new programming languages and frameworks" }
                        li { "Hiking and photographing nature" }
                        li { "Contributing to open-source projects" }
                    }
                }
                section { class: "mt-12",
                    p { class: "text-lg",
                        "Interested in collaborating or just want to chat? Feel free to reach out!"
                    }
                    div { class: "mt-4 space-x-4",
                        Link { to: Route::Home {}, class: "underline", "Contact Me" }
                        Link { to: Route::Home {}, class: "underline", "View My Projects" }
                    }
                }
            }
        }
    }
}

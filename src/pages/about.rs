use crate::{components::Hr, markdown::Markdown, Route};
use dioxus::prelude::*;
#[component]
pub fn AboutMe() -> Element {
    rsx! {
        div { class: "container mx-auto px-4 py-8",
            article { class: "prose prose-invert prose-stone prose-h2:mb-0 lg:prose-lg !max-w-none",
                h1 { "about me" }
                p {
                    "Hey, I'm George. I've been studying Bioengineering and Bioinformatics at "
                    a {
                        href: "https://fbb.msu.ru/",
                        class: "alien-link",
                        rel: "noopener noreferrer",
                        target: "_blank",
                        "FBB MSU"
                    }
                    ". Now, I live in UAE with my wonderful wife (she drew the alien, btw.) and a cat named "
                    a {
                        href: "https://teamsesh.bigcartel.com/",
                        class: "alien-link",
                        rel: "noopener noreferrer",
                        target: "_blank",
                        "Sesh"
                    }
                    " ↓"
                }
                figure { class: "max-w-prose ml-auto mr-auto block",
                    img {
                        class: "w-full h-auto aspect-[1.18/1]",
                        src: asset!("/public/sesh.avif", ImageAssetOptions::new().with_avif()),
                        alt: "Sesh the cat",
                    }
                    figcaption { "yes, he is a pirate" }
                }
                Markdown { value: r#"I like learning new stuff, playing videogames competitively, doing scientific illustrations and renders and pick up a new hobby or two every friday."# }
                p {
                    r#"This site is my digital garden.
                   I don't intend to put up paywalls, collect your data, or track your actions. 
                   It is merely a place for self-expression and experimentation. If you DO want me to collect your data,
                   please leave a signature in my "#
                    Link { to: Route::Guestbook {}, class: "alien-link", "guestbook" }
                    ". Enjoy <3"
                }
                h3 { "contacts" }
                ul {
                    li { "Email: yawner@pm.me" }
                    li {
                        a {
                            href: "https://www.linkedin.com/in/georgiy-andreev/",
                            class: "alien-link",
                            rel: "noopener noreferrer",
                            target: "_blank",
                            "LinkedIn"
                        }
                    }
                    li {
                        a {
                            href: "https://github.com/LilDojd",
                            class: "alien-link",
                            rel: "noopener noreferrer",
                            target: "_blank",
                            "GitHub"
                        }
                    }
                }
                h2 { "what i'm up to" }
                Hr { comment: "dec 2024".to_string() }
                Markdown { value: r#"
                I currently work full-time as a software engineer at [InSilico Medicine](https://insilico.com/). 
                We do some cool drug design-related stuff that involves a lot of AI. I am particularly proud
                of my contribution to the development of [INS018_055](https://www.eurekalert.org/news-releases/1048870) for the 
                treatment of IPF. I hope it gets to patients soon! Also, there is [Alchemistry](https://insilico.com/chemistry42#rec745522589).

                Also:
                - tinkering with this website
                - trying to build a molecular dynamics engine in Rust 🦀
                - getting into embedded with [RMK](https://github.com/HaoboGu/rmk)
                "# }
                h2 { "what i'm using" }
                Hr {}
                h3 { "software" }
                Markdown { value: r#"
                                This website is built with [Dioxus](https://github.com/DioxusLabs/dioxus) and 
                                [axum](https://github.com/tokio-rs/axum), and is deployed on 
                                [Fly.io](https://fly.io/).
                                
                                - python stuff in [VSCode](https://code.visualstudio.com/)
                                - everything else in [neovim](https://neovim.io/)
                                - notes: [Obsidian](https://obsidian.md/)
                                - terminal: [kitty](https://github.com/kovidgoyal/kitty)
                                
                                I try to keep my dotfiles up-to-date [here](https://github.com/LilDojd/dotfiles).
            "# }
                h3 { "hardware" }
                Markdown {
                    value: r#"
                                - Macbook: M1 MacBook Pro 16" 2021, 32GB RAM
                                - PC:
                                    - CPU: [AMD Ryzen 9 7950X](https://www.amd.com/en/products/processors/desktops/ryzen/7000-series/amd-ryzen-9-7950x.html)
                                    - MB: [ROG STRIX X670E-E](https://rog.asus.com/motherboards/rog-strix/rog-strix-x670e-e-gaming-wifi-model/)
                                    - Memory: [4xDDR5 16GB 6200MHz](https://www.corsair.com/us/en/p/memory/cmt32gx5m2x6200c36w/dominatora-platinum-rgb-32gb-2x16gb-ddr5-dram-6200mhz-c36-memory-kit-a-white-cmt32gx5m2x6200c36w) - yeah, 4x16Gb I know. but they were cheap when I was building my PC.
                                    - Storage: [SSD 980 Pro 2TB M.2](https://www.samsung.com/us/computing/memory-storage/solid-state-drives/980-pro-pcie-4-0-nvme-ssd-2tb-mz-v8p2t0b-am/) + 4Tb HDD
                                    - GPU: [Zotac RTX 4090](https://www.zotac.com/us/product/graphics_card/zotac-gaming-geforce-rtx-4090-amp-extreme-airo)
                                    - PSU: [ASUS ROG Thor 1200W Platinum](https://rog.asus.com/power-supply-units/rog-thor/rog-thor-1200p-model/)
                                    - Case: [Lian Li O11 Dynamic Evo](https://lian-li.com/product/o11-dynamic-evo/)
                                - Peripheral:
                                    - Keys: [Sofle V2](https://josefadamcik.github.io/SofleKeyboard/) and [Logitech G915 LIGHTSPEED](https://www.logitechg.com/en-ae/products/gaming-keyboards/g915-low-profile-wireless-mechanical-gaming-keyboard.html)
                                    - Monitors: [LG 32UN880-B 32](https://www.lg.com/ae/consumer-monitors/lg-32un880-b) and [LG 27GP950-B](https://www.lg.com/ae/consumer-monitors/lg-27gp950-b)
                                    - Headphones: [Sony WH-1000XM3](https://www.sony.com/en-ae/electronics/headband-headphones/wh-1000xm3)
            "#,
                }
                h2 { "other" }
                Hr {}
                ul {
                    li {
                        a {
                            href: asset!("/public/CV_GeorgyAndreev_111124.pdf"),
                            class: "alien-link",
                            target: "_blank",
                            r"CV"
                        }
                    }
                }
            }
        }
    }
}

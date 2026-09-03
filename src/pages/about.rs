use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn AboutMe() -> Element {
    rsx! {
        section { class: "prose-font flex flex-col gap-11 text-pretty text-lg leading-[1.5] text-prose xl:text-[19px]",
            div { class: "flex flex-col gap-5",
                header { class: "flex flex-col gap-2 font-recursive",
                    span { class: "label-mono", "// about" }
                    h1 { class: "heading-casual m-0 text-[30px] leading-[1.2] text-text", "about me" }
                }
                p { class: "m-0",
                    "Hey, I'm George. I've been studying Bioengineering and Bioinformatics at "
                    External { href: "https://fbb.msu.ru/", "FBB MSU" }
                    ". Now, I live in UAE with my wonderful wife (she drew the alien, btw.) and a cat named "
                    External { href: "https://teamsesh.bigcartel.com/", "Sesh" }
                    " ↓"
                }
                figure { class: "m-0 mt-1 flex items-end gap-4",
                    img { class: "block w-80 max-w-full rounded-md", src: asset!("/assets/sesh.avif"), alt: "Sesh the cat" }
                    figcaption { class: "label-mono pb-1", "yes, he is a pirate" }
                }
                p { class: "m-0", "I like learning new stuff, playing videogames competitively, doing scientific illustrations and renders and pick up a new hobby or two every friday." }
                p { class: "m-0",
                    "This site is my digital garden. I don't intend to put up paywalls, collect your data, or track your actions. It is merely a place for self-expression and experimentation. If you DO want me to collect your data, please leave a signature in my "
                    Link { to: Route::Guestbook {}, class: "link-dashed", "guestbook" }
                    ". Enjoy <3"
                }
                div { class: "mt-1 flex flex-wrap gap-2 font-recursive",
                    a { href: "mailto:yawner@pm.me", class: "pill px-[11px] py-[5px] text-[13px]", "yawner@pm.me" }
                    Contact { href: "https://www.linkedin.com/in/georgiy-andreev/", "linkedin ↗" }
                    Contact { href: "https://github.com/LilDojd", "github ↗" }
                    a { href: asset!("/assets/CV_GeorgyAndreev_042025.pdf"), target: "_blank", rel: "noopener noreferrer", class: "pill px-[11px] py-[5px] text-[13px]", "cv (pdf) ↗" }
                }
            }

            div { class: "flex flex-col gap-5",
                span { class: "label-mono", "// what i'm up to" }
                LabelRow { label: "apr 2025", accent: true,
                    p { class: "m-0",
                        "I am excited to share that I have joined "
                        External { href: "https://genbio.ai/", "GenBio AI" }
                        " as a Research Engineer on a quest to build first-in-class foundational models for biology! Life is great."
                    }
                }
                div { class: "text-muted",
                    LabelRow { label: "dec 2024",
                        p { class: "m-0",
                            "I currently work full-time as a software engineer at "
                            External { href: "https://insilico.com/", "InSilico Medicine" }
                            ". We do some cool drug design-related stuff that involves a lot of AI. I am particularly proud of my contribution to the development of "
                            External { href: "https://www.eurekalert.org/news-releases/1048870", "INS018_055" }
                            " for the treatment of IPF. I hope it gets to patients soon! Also, there is "
                            External { href: "https://insilico.com/chemistry42#rec745522589", "Alchemistry" }
                            "."
                        }
                    }
                }
                LabelRow { label: "also", small: true,
                    div { class: "flex flex-col gap-1",
                        span { "tinkering with this website" }
                        span { "trying to build a molecular dynamics engine in Rust 🦀" }
                        span { "getting into embedded with " External { href: "https://github.com/HaoboGu/rmk", "RMK" } }
                    }
                }
            }

            div { class: "flex flex-col gap-5 text-[15px]",
                span { class: "label-mono", "// what i'm using" }
                LabelRow { label: "software", small: true,
                    div { class: "flex flex-col gap-1.5",
                        p { class: "m-0 mb-1.5",
                            "This website is built with " External { href: "https://github.com/DioxusLabs/dioxus", "Dioxus" }
                            " and " External { href: "https://github.com/tokio-rs/axum", "axum" }
                            ", and is deployed on " External { href: "https://fly.io/", "Fly.io" } "."
                        }
                        span { "python stuff in " External { href: "https://code.visualstudio.com/", "VSCode" } }
                        span { "everything else in " External { href: "https://neovim.io/", "neovim" } }
                        span { "notes: " External { href: "https://obsidian.md/", "Obsidian" } }
                        span { "terminal: " External { href: "https://github.com/kovidgoyal/kitty", "kitty" } }
                        p { class: "m-0 mt-1.5", "I try to keep my dotfiles up-to-date " External { href: "https://github.com/LilDojd/dotfiles", "here" } "." }
                    }
                }
                LabelRow { label: "hardware", small: true,
                    div { class: "grid grid-cols-[84px_1fr] gap-x-3.5 gap-y-1.5",
                        HardwareLabel { "macbook" } span { "M1 MacBook Pro 16\" 2021, 32GB RAM" }
                        HardwareLabel { "cpu" } HardwareLink { href: "https://www.amd.com/en/products/processors/desktops/ryzen/7000-series/amd-ryzen-9-7950x.html", "AMD Ryzen 9 7950X" }
                        HardwareLabel { "mb" } HardwareLink { href: "https://rog.asus.com/motherboards/rog-strix/rog-strix-x670e-e-gaming-wifi-model/", "ROG STRIX X670E-E" }
                        HardwareLabel { "memory" } span { HardwareLink { href: "https://www.corsair.com/us/en/p/memory/cmt32gx5m2x6200c36w/dominatora-platinum-rgb-32gb-2x16gb-ddr5-dram-6200mhz-c36-memory-kit-a-white-cmt32gx5m2x6200c36w", "4xDDR5 16GB 6200MHz" } span { class: "text-label", " — yeah, 4x16Gb I know. but they were cheap when I was building my PC." } }
                        HardwareLabel { "storage" } span { HardwareLink { href: "https://www.samsung.com/us/computing/memory-storage/solid-state-drives/980-pro-pcie-4-0-nvme-ssd-2tb-mz-v8p2t0b-am/", "SSD 980 Pro 2TB M.2" } " + 4Tb HDD" }
                        HardwareLabel { "gpu" } HardwareLink { href: "https://www.zotac.com/us/product/graphics_card/zotac-gaming-geforce-rtx-4090-amp-extreme-airo", "Zotac RTX 4090" }
                        HardwareLabel { "psu" } HardwareLink { href: "https://rog.asus.com/power-supply-units/rog-thor/rog-thor-1200p-model/", "ASUS ROG Thor 1200W Platinum" }
                        HardwareLabel { "case" } HardwareLink { href: "https://lian-li.com/product/o11-dynamic-evo/", "Lian Li O11 Dynamic Evo" }
                        HardwareLabel { "keys" } span { HardwareLink { href: "https://josefadamcik.github.io/SofleKeyboard/", "Sofle V2" } " and " HardwareLink { href: "https://www.logitechg.com/en-ae/products/gaming-keyboards/g915-low-profile-wireless-mechanical-gaming-keyboard.html", "Logitech G915 LIGHTSPEED" } }
                        HardwareLabel { "monitors" } span { HardwareLink { href: "https://www.lg.com/ae/consumer-monitors/lg-32un880-b", "LG 32UN880-B 32" } " and " HardwareLink { href: "https://www.lg.com/ae/consumer-monitors/lg-27gp950-b", "LG 27GP950-B" } }
                        HardwareLabel { "headphones" } HardwareLink { href: "https://www.sony.com/en-ae/electronics/headband-headphones/wh-1000xm3", "Sony WH-1000XM3" }
                    }
                }
            }
        }
    }
}

#[component]
fn External(href: &'static str, children: Element) -> Element {
    rsx! { a { href, target: "_blank", rel: "noopener noreferrer", class: "quiet-link", {children} } }
}

#[component]
fn Contact(href: &'static str, children: Element) -> Element {
    rsx! { a { href, target: "_blank", rel: "noopener noreferrer", class: "pill px-[11px] py-[5px] text-[13px]", {children} } }
}

#[component]
fn LabelRow(
    label: &'static str,
    children: Element,
    accent: Option<bool>,
    small: Option<bool>,
) -> Element {
    rsx! {
        div { class: if small.unwrap_or(false) { "grid grid-cols-[60px_1fr] gap-4 text-[15px] md:grid-cols-[72px_1fr]" } else { "grid grid-cols-[60px_1fr] gap-4 md:grid-cols-[72px_1fr]" },
            span { class: if accent.unwrap_or(false) { "label-mono pt-1 text-accent" } else { "label-mono pt-1" }, {label.to_string()} }
            {children}
        }
    }
}

#[component]
fn HardwareLabel(children: Element) -> Element {
    rsx! { span { class: "label-mono pt-[3px]", {children} } }
}

#[component]
fn HardwareLink(href: &'static str, children: Element) -> Element {
    rsx! { a { href, target: "_blank", rel: "noopener noreferrer", class: "text-secondary no-underline hover:text-accent", {children} } }
}

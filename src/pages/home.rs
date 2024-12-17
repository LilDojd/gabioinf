use crate::Route;
use async_std::task;
use dioxus::prelude::*;
use rand::Rng;
const TYPING_MILLIS: u64 = 2000;
const BLINK_MILLIS: u64 = 3500;
const ERROR_CHANCE: f64 = 0.03;
#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "container md:pt-8",
            div { class: "flex flex-col md:flex-row items-center justify-between gap-8",
                LeftColumn {}
                RightColumn {}
            }
        }
    }
}
#[component]
fn LeftColumn() -> Element {
    let mut visible_text = use_signal(String::new);
    let mut hide_cursor = use_signal(|| false);
    let full_text = "Hey, I'm George";
    let base_interval = TYPING_MILLIS / full_text.len() as u64;
    let _text_animation = use_effect(move || {
        spawn(async move {
            let mut rng = rand::thread_rng();
            let mut current_index = 0;
            while current_index < full_text.len() {
                let jitter = rng.gen_range(-10..=10);
                let interval = (base_interval as i64 + jitter).max(50) as u64;
                task::sleep(std::time::Duration::from_millis(interval)).await;
                if rng.gen_bool(ERROR_CHANCE) && current_index > 0 {
                    let mistake_char = (rng.gen_range(b'a'..=b'z') as char).to_string();
                    visible_text.set(format!(
                        "{}{}",
                        full_text.chars().take(current_index).collect::<String>(),
                        mistake_char,
                    ));
                    task::sleep(std::time::Duration::from_millis(rng.gen_range(100..300))).await;
                    visible_text.set(full_text.chars().take(current_index).collect());
                    task::sleep(std::time::Duration::from_millis(rng.gen_range(20..100))).await;
                } else {
                    current_index += 1;
                    visible_text.set(full_text.chars().take(current_index).collect());
                }
            }
            task::sleep(std::time::Duration::from_millis(200)).await;
            let blink_start = instant::Instant::now();
            while blink_start.elapsed().as_millis() < BLINK_MILLIS as u128 {
                task::sleep(std::time::Duration::from_millis(500)).await;
                hide_cursor.toggle();
            }
            hide_cursor.set(true);
        });
    });
    let links = vec![
        (
            "https://www.linkedin.com/in/georgiy-andreev".to_string(),
            "follow me on linkedin".to_string(),
        ),
        (
            "https://github.com/LilDojd".to_string(),
            "i have some stuff on github".to_string(),
        ),
        (
            "https://buymeacoffee.com/yawner".to_string(),
            "feeling generou$?".to_string(),
        ),
        (
            "https://cal.com/yawner".to_string(),
            "fancy a chat?".to_string(),
        ),
    ];
    rsx! {
        div { class: "w-full md:w-1/2 space-y-6",
            h1 { class: "text-3xl sm:text-4xl md:text-5xl font-bold text-stone-100",
                "{visible_text}"
                span {
                    class: "text-alien-green",
                    class: if *hide_cursor.read() { "invisible" },
                    "█"
                }
            }
            IntroText {}
            SaucerDivider {}
            LinkButtons { links }
        }
    }
}
#[component]
fn RightColumn() -> Element {
    rsx! {
        div { class: "w-full md:w-1/2 text-left", id: "alien-container",
            video {
                id: "alien-video",
                class: "w-full h-auto object-cover aspect-square",
                playsinline: true,
                autoplay: true,
                muted: true,
                r#loop: "false",
                source {
                    src: asset!("/public/alien_white.webm"),
                    r#type: "video/webm",
                }
                source {
                    src: asset!("/public/alien_white.mov"),
                    r#type: "video/mp4;codecs=hvc1",
                }
                img {
                    src: asset!("/public/alien_white.png", ImageAssetOptions::new().with_avif()),
                    alt: "Fallback image",
                }
            }
        }
    }
}
#[component]
fn IntroText() -> Element {
    rsx! {
        div { class: "text-lg text-stone-300",
            p { class: "mb-4", "I'm a bioinformatician and a developer." }
            p { class: "mb-6",
                "You can use this website to read my "
                Link { to: Route::Blog {}, class: "alien-link", "random rambles" }
                ", learn more "
                Link { to: Route::AboutMe {}, class: "alien-link", "about me" }
                " and "
                Link { to: Route::Guestbook {}, class: "alien-link", "sign my guestbook" }
                " <3"
            }
        }
    }
}
#[derive(Props, PartialEq, Clone)]
struct LinkButtonsProps {
    links: Vec<(String, String)>,
}
#[component]
fn LinkButtons(props: LinkButtonsProps) -> Element {
    rsx! {
        div { class: "flex flex-col space-y-2 mt-6 items-start",
            {
                props
                    .links
                    .iter()
                    .map(|(url, text)| {
                        rsx! {
                            a {
                                href: "{url}",
                                rel: "noopener noreferrer",
                                target: "_blank",
                                class: "inline-block",
                                span { class: "text-stone-400 text-sm flex gap-1 my-2 items-center justify-center hover:text-stone-100 cursor-pointer",
                                    svg {
                                        class: "w-5 h-5 ml-1",
                                        fill: "none",
                                        view_box: "0 0 24 24",
                                        xmlns: "http://www.w3.org/2000/svg",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_width: "2",
                                            stroke_linejoin: "round",
                                            d: "m8 16 8-8m0 0h-6m6 0v6",
                                            stroke: "currentColor",
                                        }
                                    }
                                    span { "{text}" }
                                }
                            }
                        }
                    })
            }
        }
    }
}
#[component]
fn SaucerDivider() -> Element {
    rsx! {
        div { class: "h-4",
            svg {
                fill: "none",
                "viewBox": "0 0 432 38",
                width: "80",
                height: "16",
                "aria-hidden": "true",
                class: "mt-10 mb-2 text-stone-400",
                path {
                    d: "M402.74 37.5899C390.193 37.5899 374.767 21.3129 374.111 20.6249C367.068 12.4335 359.943 5.14795 349.463 5.14795C337.975 5.14795 324.479 20.406 324.338 20.558L323.17 21.8313C315.729 29.9329 308.701 37.5893 296.186 37.5893C283.639 37.5893 268.213 21.3123 267.557 20.6243C260.514 12.4329 253.389 5.14734 242.909 5.14734C231.421 5.14734 217.925 20.4053 217.784 20.5573L216.683 21.7175C208.186 30.5847 201.48 37.5885 189.636 37.5885C177.085 37.5885 161.656 21.3115 161.007 20.6235C153.96 12.4321 146.831 5.14655 136.359 5.14655C124.871 5.14655 111.375 20.4045 111.234 20.5565L110.054 21.8417C102.62 29.9394 95.5889 37.5837 83.0769 37.5837C70.5259 37.5837 55.0969 21.3067 54.4479 20.6187C47.401 12.4273 40.2719 5.14175 29.7999 5.14175C19.3699 5.14175 9.86587 10.8722 4.98787 20.0987C4.3824 21.2549 2.94488 21.6964 1.78478 21.087C0.628579 20.4698 0.187069 19.0401 0.800389 17.8839C6.50349 7.10691 17.6124 0.403931 29.7964 0.403931C42.2694 0.403931 50.5504 8.82583 57.9644 17.4469C61.941 21.6774 74.3554 32.8419 83.0734 32.8419C93.5074 32.8419 99.2644 26.5724 106.557 18.6349L107.702 17.3888C108.268 16.7404 122.733 0.404816 136.35 0.404816C148.823 0.404816 157.104 8.82671 164.518 17.4478C168.494 21.6783 180.909 32.8428 189.627 32.8428C199.447 32.8428 204.943 27.1123 213.256 18.4368L214.295 17.3509C214.83 16.7337 229.295 0.401917 242.908 0.401917C255.388 0.401917 263.67 8.82382 271.076 17.4449C275.053 21.6676 287.467 32.8359 296.185 32.8359C306.623 32.8359 312.388 26.5625 319.685 18.6129L320.822 17.3785C321.388 16.7301 335.853 0.394531 349.463 0.394531C361.943 0.394531 370.225 8.81643 377.631 17.4375C381.607 21.6602 394.022 32.8285 402.74 32.8285C412.744 32.8285 422.06 27.4379 427.064 18.7625C427.716 17.6258 429.161 17.2313 430.302 17.8914C431.435 18.5438 431.822 19.993 431.173 21.1258C425.321 31.2898 414.427 37.5908 402.739 37.5908L402.74 37.5899Z",
                    fill: "currentColor",
                }
            }
        }
    }
}

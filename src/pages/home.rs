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
    let mut animate = use_signal(|| false);
    use_effect(move || {
        if *animate.read() {
            let _ = eval(
                r#"var el = document.getElementById("divider-svg");
                   if (el.contentDocument && el.contentDocument.defaultView.KeyshapeJS) {
                       var ks = el.contentDocument.defaultView.KeyshapeJS;
                       ks.globalPlay();
                }"#,
            );
        }
    });
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
            animate.set(true);
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
            SaucerDivier { animate }
            LinkButtons { links }
        }
    }
}
#[component]
fn RightColumn() -> Element {
    rsx! {
        div { class: "w-full md:w-1/2 text-left",
            video {
                class: "w-full h-auto object-cover",
                playsinline: true,
                autoplay: true,
                muted: true,
                r#loop: "false",
                src: "/alien_white.webm",
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
                Link { to: Route::Home {}, class: "alien-link", "random rambles" }
                ", learn more "
                Link { to: Route::AboutMe {}, class: "alien-link", "about me" }
                " and "
                Link { to: Route::Home {}, class: "alien-link", "sign my guestbook" }
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
#[rustfmt::skip]
fn SaucerDivier(animate: Signal<bool>) -> Element {
    let random_id = rand::random::<u32>();
    rsx! {
        object { data : format!("{}?r={}", asset!("public/saucer_divider.svg"),
        random_id), id : "divider-svg", alt : "Flying saucer divider", r#type :
        "image/svg+xml", class : "h-4", onload : | _ | { _ =
        eval(r#"var el = document.getElementById("divider-svg");
                               if (el.contentDocument && el.contentDocument.defaultView.KeyshapeJS) {
                                   var ks = el.contentDocument.defaultView.KeyshapeJS;
                                   ks.globalPause();
                            }"#,)
        }, }
    }
}

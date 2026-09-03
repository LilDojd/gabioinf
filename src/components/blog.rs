use dioxus::prelude::*;

#[component]
pub fn GcCalculator() -> Element {
    let mut sequence = use_signal(String::new);
    let stats = sequence_stats(&sequence());

    rsx! {
        section {
            class: "my-8 rounded-md border border-card bg-surface p-5",
            aria_label: "GC calculator",
            h2 { class: "heading-casual mt-0 text-xl text-text", "GC calculator" }
            label { class: "label-mono mt-4 block",
                "DNA or RNA sequence"
                textarea {
                    class: "mt-2 min-h-32 w-full rounded-md border border-card bg-code p-3 font-recursive text-sm text-text focus:border-accent focus:outline-none",
                    placeholder: "ACGTACGT",
                    spellcheck: "false",
                    value: sequence,
                    oninput: move |event| sequence.set(event.value()),
                }
            }
            output { class: "mt-4 block text-secondary", aria_live: "polite",
                match stats {
                    Ok(Some((bases, gc))) => rsx! {
                        strong { class: "text-accent", "{gc:.1}% GC" }
                        span { class: "ml-2 text-sm text-label", "across {bases} bases" }
                    },
                    Ok(None) => rsx! { span { class: "text-label", "Enter a sequence to calculate its GC content." } },
                    Err(character) => rsx! { span { class: "text-mars", "Unsupported character: {character}" } },
                }
            }
        }
    }
}

/// A highlighted code block with a "copy" button (clipboard access goes through
/// web-sys, so there is no JavaScript involved).
#[component]
pub fn CodeBlock(
    language: Option<&'static str>,
    html: &'static str,
    source: &'static str,
) -> Element {
    let mut copied = use_signal(|| false);
    let label = language.unwrap_or("text");

    rsx! {
        figure { class: "code-block",
            figcaption { class: "code-block-bar",
                span { {label} }
                button {
                    r#type: "button",
                    class: "code-copy",
                    onclick: move |_| {
                        copy_to_clipboard(source);
                        copied.set(true);
                        spawn(async move {
                            wasmtimer::tokio::sleep(std::time::Duration::from_millis(1500)).await;
                            copied.set(false);
                        });
                    },
                    if copied() { "copied" } else { "copy" }
                }
            }
            // Highlighted at build time from trusted Markdown in `content/blog/`.
            pre { tabindex: "0",
                code { class: if let Some(language) = language { "language-{language}" }, dangerous_inner_html: html }
            }
        }
    }
}

fn copy_to_clipboard(text: &str) {
    #[cfg(feature = "web")]
    if let Some(window) = web_sys::window() {
        // The returned promise resolves in the background; a failed copy just leaves the clipboard alone.
        let _ = window.navigator().clipboard().write_text(text);
    }
    #[cfg(not(feature = "web"))]
    let _ = text;
}

#[component]
pub fn BlogVideo(src: &'static str, title: Option<&'static str>) -> Element {
    let label = title.unwrap_or("Embedded video");

    rsx! {
        figure { class: "my-8",
            video {
                class: "w-full rounded-md border border-card bg-code",
                controls: true,
                preload: "metadata",
                aria_label: label,
                source { src }
                "Your browser does not support embedded video."
            }
            if let Some(title) = title {
                figcaption { class: "label-mono mt-2 text-center", {title} }
            }
        }
    }
}

fn sequence_stats(sequence: &str) -> Result<Option<(usize, f64)>, char> {
    let mut bases = 0;
    let mut gc_bases = 0;
    for character in sequence
        .chars()
        .filter(|character| !character.is_whitespace())
    {
        match character.to_ascii_uppercase() {
            'G' | 'C' => gc_bases += 1,
            'A' | 'T' | 'U' | 'N' => {}
            invalid => return Err(invalid),
        }
        bases += 1;
    }

    Ok((bases > 0).then(|| (bases, gc_bases as f64 / bases as f64 * 100.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_gc_content_and_validates_input() {
        assert_eq!(sequence_stats("AGCN"), Ok(Some((4, 50.0))));
        assert_eq!(sequence_stats(" \n"), Ok(None));
        assert_eq!(sequence_stats("ACX"), Err('X'));
    }
}

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

/// A code viewer: line numbers, click / shift-click on numbers to select a line
/// range (mirrored into the URL as `#L3-L7`, GitHub-style, and restored on load),
/// author-highlighted lines from the fence, a wrap toggle and a copy button.
/// Everything talks to the browser through web-sys; there is no JavaScript.
#[component]
pub fn CodeBlock(
    language: Option<&'static str>,
    title: Option<&'static str>,
    lines: &'static [&'static str],
    highlighted: &'static [usize],
    source: &'static str,
) -> Element {
    let mut selection = use_signal(|| None::<LineRange>);
    let mut selection_anchor = use_signal(|| None::<usize>);
    let mut wrap = use_signal(|| false);
    let mut copied = use_signal(|| false);

    // One block per page may be deep-linked; the first block claims a matching hash.
    use_effect(move || {
        if let Some(range) = location_hash().and_then(|hash| LineRange::parse(&hash))
            && range.end <= lines.len()
            && selection().is_none()
        {
            selection_anchor.set(Some(range.start));
            selection.set(Some(range));
        }
    });

    let mut select = move |line: usize, extend: bool| {
        let anchor = if extend {
            selection_anchor().unwrap_or_else(|| {
                selection_anchor.set(Some(line));
                line
            })
        } else {
            selection_anchor.set(Some(line));
            line
        };
        let range = LineRange::between(anchor, line);
        selection.set(Some(range));
        replace_hash(&range.to_hash());
    };

    rsx! {
        figure { class: if wrap() { "code-block code-wrap" } else { "code-block" },
            figcaption { class: "code-block-bar",
                span { class: "code-title", {title.or(language).unwrap_or("text")} }
                span { class: "code-actions",
                    if let Some(range) = selection() {
                        span { class: "code-selection", "{range}" }
                        button {
                            r#type: "button",
                            class: "code-action",
                            onclick: move |_| {
                                selection.set(None);
                                selection_anchor.set(None);
                                replace_hash("");
                            },
                            "clear"
                        }
                    }
                    button {
                        r#type: "button",
                        class: "code-action",
                        aria_pressed: wrap(),
                        onclick: move |_| wrap.toggle(),
                        "wrap"
                    }
                    button {
                        r#type: "button",
                        class: "code-action",
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
            }
            pre { tabindex: "0",
                code { class: if let Some(language) = language { "language-{language}" },
                    for (index, line) in lines.iter().enumerate() {
                        {
                            let number = index + 1;
                            let selected = selection().is_some_and(|range| range.contains(number));
                            let emphasised = highlighted.contains(&number);
                            rsx! {
                                span {
                                    key: "{number}",
                                    class: "code-line",
                                    class: if selected { "is-selected" },
                                    class: if emphasised { "is-highlighted" },
                                    // Line numbers are buttons, so text selection and copy never include them.
                                    button {
                                        r#type: "button",
                                        class: "code-line-number",
                                        title: "select line {number} (shift-click for a range)",
                                        aria_pressed: selected,
                                        onclick: move |event| select(number, event.modifiers().shift()),
                                        "{number}"
                                    }
                                    // Highlighted at build time from trusted Markdown in `content/blog/`.
                                    span { class: "code-line-text", dangerous_inner_html: *line }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// An inclusive 1-based line range, formatted like GitHub (`L3` or `L3-L7`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LineRange {
    start: usize,
    end: usize,
}

impl LineRange {
    fn between(first: usize, second: usize) -> Self {
        Self {
            start: first.min(second),
            end: first.max(second),
        }
    }

    fn contains(self, line: usize) -> bool {
        (self.start..=self.end).contains(&line)
    }

    fn to_hash(self) -> String {
        format!("#{self}")
    }

    fn parse(hash: &str) -> Option<Self> {
        let hash = hash.trim_start_matches('#');
        let (start, end) = match hash.split_once('-') {
            Some((start, end)) => (start, end),
            None => (hash, hash),
        };
        let line = |text: &str| {
            text.strip_prefix('L')?
                .parse::<usize>()
                .ok()
                .filter(|line| *line >= 1)
        };
        let (start, end) = (line(start)?, line(end)?);
        (start <= end).then_some(Self { start, end })
    }
}

impl std::fmt::Display for LineRange {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(formatter, "L{}", self.start)
        } else {
            write!(formatter, "L{}-L{}", self.start, self.end)
        }
    }
}

fn location_hash() -> Option<String> {
    #[cfg(feature = "web")]
    {
        web_sys::window()?
            .location()
            .hash()
            .ok()
            .filter(|hash| !hash.is_empty())
    }
    #[cfg(not(feature = "web"))]
    None
}

/// Updates the URL fragment without adding history entries or scrolling.
fn replace_hash(hash: &str) {
    #[cfg(feature = "web")]
    if let Some(window) = web_sys::window()
        && let Ok(path) = window.location().pathname()
    {
        let url = format!("{path}{hash}");
        let _ = window.history().and_then(|history| {
            history.replace_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(&url))
        });
    }
    #[cfg(not(feature = "web"))]
    let _ = hash;
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
    fn line_ranges_round_trip_through_the_url_hash() {
        let range = LineRange::between(7, 3);
        assert_eq!(range, LineRange { start: 3, end: 7 });
        assert_eq!(range.to_hash(), "#L3-L7");
        assert_eq!(LineRange::parse("#L3-L7"), Some(range));
        assert_eq!(LineRange::parse("#L4"), Some(LineRange::between(4, 4)));
        assert_eq!(LineRange::parse("#L7-L3"), None);
        assert_eq!(LineRange::parse("#comments"), None);
    }

    #[test]
    fn calculates_gc_content_and_validates_input() {
        assert_eq!(sequence_stats("AGCN"), Ok(Some((4, 50.0))));
        assert_eq!(sequence_stats(" \n"), Ok(None));
        assert_eq!(sequence_stats("ACX"), Err('X'));
    }
}

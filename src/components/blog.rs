use dioxus::prelude::*;

#[component]
pub fn GcCalculator() -> Element {
    let mut sequence = use_signal(String::new);
    let stats = sequence_stats(&sequence());

    rsx! {
        section {
            class: "my-8 rounded-lg border border-onyx bg-jet p-6",
            aria_label: "GC calculator",
            h2 { class: "mt-0 text-xl font-semibold text-stone-100", "GC calculator" }
            label { class: "mt-4 block text-sm text-stone-300",
                "DNA or RNA sequence"
                textarea {
                    class: "mt-2 min-h-32 w-full rounded-md border border-onyx bg-nasty-black p-3 font-mono text-sm text-stone-100 focus:border-alien-green focus:outline-none",
                    placeholder: "ACGTACGT",
                    spellcheck: "false",
                    value: sequence,
                    oninput: move |event| sequence.set(event.value()),
                }
            }
            output { class: "mt-4 block text-stone-200", aria_live: "polite",
                match stats {
                    Ok(Some((bases, gc))) => rsx! {
                        strong { class: "text-alien-green", "{gc:.1}% GC" }
                        span { class: "ml-2 text-sm text-stone-400", "across {bases} bases" }
                    },
                    Ok(None) => rsx! { span { class: "text-stone-400", "Enter a sequence to calculate its GC content." } },
                    Err(character) => rsx! { span { class: "text-coral", "Unsupported character: {character}" } },
                }
            }
        }
    }
}

#[component]
pub fn BlogVideo(src: &'static str, title: Option<&'static str>) -> Element {
    let label = title.unwrap_or("Embedded video");

    rsx! {
        figure { class: "my-8",
            video {
                class: "w-full rounded-lg border border-onyx bg-black",
                controls: true,
                preload: "metadata",
                aria_label: label,
                source { src }
                "Your browser does not support embedded video."
            }
            if let Some(title) = title {
                figcaption { class: "mt-2 text-center text-sm text-stone-400", {title} }
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

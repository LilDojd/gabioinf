use dioxus::prelude::*;
use rand::RngExt;
use std::time::Duration;
use wasmtimer::tokio::sleep;

// Let the page settle briefly before the greeting starts.
const START_DELAY_MS: u64 = 300;
// Spread normal keystrokes across roughly two seconds.
const TOTAL_TYPING_MS: u64 = 2_000;
// Keep the rhythm human without allowing an imperceptibly short delay.
const JITTER_MS: i64 = 10;
const MIN_KEYSTROKE_MS: i64 = 50;
// Occasionally show and correct a plausible typo.
const TYPO_CHANCE: f64 = 0.04;
const TYPO_VISIBLE_MS: std::ops::RangeInclusive<u64> = 120..=300;
const TYPO_DELETE_MS: std::ops::RangeInclusive<u64> = 60..=120;
// Blink seven times at half-second intervals before hiding the cursor.
const CURSOR_BLINK_MS: u64 = 500;
const CURSOR_TOGGLES: usize = 7;

#[component]
pub fn Typewriter(
    text: &'static str,
    generation: u32,
    mut completed: Signal<Option<u32>>,
) -> Element {
    let resolved = *completed.peek() == Some(generation);
    let mut typed = use_signal(move || {
        if resolved {
            text.to_string()
        } else {
            String::new()
        }
    });
    let mut cursor_visible = use_signal(move || !resolved);
    let mut active_generation = use_signal(|| generation);

    use_effect(use_reactive!(|text, generation| {
        active_generation.set(generation);
        if *completed.peek() == Some(generation) {
            typed.set(text.to_string());
            cursor_visible.set(false);
            return;
        }
        typed.set(String::new());
        cursor_visible.set(true);

        spawn(async move {
            sleep(Duration::from_millis(START_DELAY_MS)).await;
            let characters = text.chars().collect::<Vec<_>>();
            let mut rng = rand::rng();

            for (index, character) in characters.iter().copied().enumerate() {
                if active_generation() != generation {
                    return;
                }
                if index > 0 && rng.random_bool(TYPO_CHANCE) {
                    typed
                        .write()
                        .push(char::from(rng.random_range(b'a'..=b'z')));
                    sleep(Duration::from_millis(rng.random_range(TYPO_VISIBLE_MS))).await;
                    if active_generation() != generation {
                        return;
                    }
                    typed.write().pop();
                    sleep(Duration::from_millis(rng.random_range(TYPO_DELETE_MS))).await;
                    if active_generation() != generation {
                        return;
                    }
                }

                typed.write().push(character);
                sleep(typing_delay(
                    characters.len(),
                    rng.random_range(-JITTER_MS..=JITTER_MS),
                ))
                .await;
            }

            if active_generation() != generation {
                return;
            }
            completed.set(Some(generation));

            for _ in 0..CURSOR_TOGGLES {
                sleep(Duration::from_millis(CURSOR_BLINK_MS)).await;
                if active_generation() != generation {
                    return;
                }
                cursor_visible.toggle();
            }
            cursor_visible.set(false);
        });
    }));

    rsx! {
        "{typed}"
        span {
            class: if cursor_visible() { "ml-0.5 inline-block h-[.95em] w-[.5em] translate-y-[.12em] bg-accent" } else { "ml-0.5 inline-block h-[.95em] w-[.5em] translate-y-[.12em] bg-accent opacity-0" },
        }
    }
}

fn typing_delay(character_count: usize, jitter_millis: i64) -> Duration {
    let count = u64::try_from(character_count.max(1)).expect("character count fits in u64");
    let base = i64::try_from(TOTAL_TYPING_MS / count).expect("typing delay fits in i64");
    let millis = u64::try_from((base + jitter_millis).max(MIN_KEYSTROKE_MS))
        .expect("the minimum typing delay is non-negative");
    Duration::from_millis(millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_delay_uses_text_length_jitter_and_a_minimum() {
        assert_eq!(typing_delay(10, 0), Duration::from_millis(200));
        assert_eq!(typing_delay(10, -10), Duration::from_millis(190));
        assert_eq!(typing_delay(100, -10), Duration::from_millis(50));
        assert_eq!(typing_delay(0, 0), Duration::from_millis(2_000));
    }
}

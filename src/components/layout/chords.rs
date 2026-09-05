//! The keyboard "language" of the site, kept free of DOM types so it can be
//! unit-tested: `g` + letter chords, vim-style `10j` counts, and the `sesh` egg.

use crate::Route;
use std::collections::VecDeque;

pub(super) const CHORD_TIMEOUT_MILLIS: f64 = 900.0;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Key {
    Escape,
    MetaK,
    Slash,
    Question,
    Character(char),
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Direction {
    Down,
    Up,
}

#[derive(Clone, PartialEq)]
pub(super) enum Action {
    Navigate(Route),
    TogglePalette,
    OpenPalette,
    ToggleHelp,
    CloseOverlays,
    Scroll {
        direction: Direction,
        count: Option<u32>,
    },
    Sesh,
}

#[derive(Default)]
pub(super) struct Chords {
    pub now_millis: f64,
    pending_g: Option<f64>,
    count: Option<(u32, f64)>,
    sesh: VecDeque<char>,
}

impl Chords {
    pub fn handle(&mut self, key: &Key) -> Option<Action> {
        self.handle_at(key, self.now_millis)
    }

    fn handle_at(&mut self, key: &Key, now_millis: f64) -> Option<Action> {
        match key {
            Key::Escape => return Some(Action::CloseOverlays),
            Key::MetaK => return Some(Action::TogglePalette),
            Key::Slash => return Some(Action::OpenPalette),
            Key::Question => return Some(Action::ToggleHelp),
            Key::Character(character) if character.is_ascii_digit() => {
                let digit = character
                    .to_digit(10)
                    .expect("an ASCII digit has a decimal value");
                let previous = self
                    .count
                    .filter(|(_, updated)| now_millis - updated <= CHORD_TIMEOUT_MILLIS)
                    .map_or(0, |(count, _)| count);
                self.count = Some((
                    previous.saturating_mul(10).saturating_add(digit),
                    now_millis,
                ));
                return None;
            }
            Key::Character('j' | 'k') => {
                let direction = if matches!(key, Key::Character('j')) {
                    Direction::Down
                } else {
                    Direction::Up
                };
                let count = self
                    .count
                    .take()
                    .filter(|(_, updated)| now_millis - updated <= CHORD_TIMEOUT_MILLIS)
                    .map(|(count, _)| count);
                self.pending_g = None;
                return Some(Action::Scroll { direction, count });
            }
            Key::Character(_) | Key::Other => {
                self.count = None;
            }
        }

        let Key::Character(character) = key else {
            self.pending_g = None;
            return None;
        };

        if self
            .pending_g
            .take()
            .is_some_and(|started| now_millis - started <= CHORD_TIMEOUT_MILLIS)
            && let Some(route) = route_for_key(*character)
        {
            return Some(Action::Navigate(route));
        }
        if *character == 'g' {
            self.pending_g = Some(now_millis);
        }

        self.sesh.push_back(*character);
        if self.sesh.len() > 4 {
            self.sesh.pop_front();
        }
        if self.sesh.iter().copied().eq("sesh".chars()) {
            self.sesh.clear();
            return Some(Action::Sesh);
        }
        None
    }
}

pub(super) fn route_for_key(key: char) -> Option<Route> {
    match key {
        'h' => Some(Route::Home {}),
        'b' => Some(Route::Blog {}),
        'p' => Some(Route::Projects {}),
        'a' => Some(Route::AboutMe {}),
        'g' => Some(Route::Guestbook {}),
        'v' => Some(Route::NotFound {
            route: vec!["void".to_string()],
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_overlay_keys_to_actions() {
        let mut chords = Chords::default();

        assert!(matches!(
            chords.handle(&Key::Escape),
            Some(Action::CloseOverlays)
        ));
        assert!(matches!(
            chords.handle(&Key::MetaK),
            Some(Action::TogglePalette)
        ));
        assert!(matches!(
            chords.handle(&Key::Slash),
            Some(Action::OpenPalette)
        ));
        assert!(matches!(
            chords.handle(&Key::Question),
            Some(Action::ToggleHelp)
        ));
        assert!(chords.handle(&Key::Other).is_none());
    }

    #[test]
    fn handles_counts_navigation_and_sesh_without_mixing_digits() {
        let mut chords = Chords::default();
        let now = 0.0;

        assert!(chords.handle_at(&Key::Character('1'), now).is_none());
        assert!(chords.handle_at(&Key::Character('0'), now).is_none());
        assert!(
            chords.handle_at(&Key::Character('j'), now)
                == Some(Action::Scroll {
                    direction: Direction::Down,
                    count: Some(10),
                })
        );

        assert!(chords.handle_at(&Key::Character('g'), now).is_none());
        assert!(matches!(
            chords.handle_at(&Key::Character('b'), now),
            Some(Action::Navigate(Route::Blog {}))
        ));

        for key in ['s', 'e', '1', 's', 'h'] {
            let action = chords.handle_at(&Key::Character(key), now);
            if key == 'h' {
                assert!(action == Some(Action::Sesh));
            }
        }
    }

    #[test]
    fn expires_count_prefixes() {
        let mut chords = Chords::default();
        let now = 0.0;
        chords.handle_at(&Key::Character('4'), now);

        assert!(
            chords.handle_at(&Key::Character('k'), now + CHORD_TIMEOUT_MILLIS + 1.0)
                == Some(Action::Scroll {
                    direction: Direction::Up,
                    count: None,
                })
        );
    }
}

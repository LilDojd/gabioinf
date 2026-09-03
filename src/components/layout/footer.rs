use crate::Route;
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Planet {
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

impl Planet {
    const fn name(self) -> &'static str {
        match self {
            Self::Mercury => "mercury",
            Self::Venus => "venus",
            Self::Earth => "earth",
            Self::Mars => "mars",
            Self::Jupiter => "jupiter",
            Self::Saturn => "saturn",
            Self::Uranus => "uranus",
            Self::Neptune => "neptune",
        }
    }

    const fn class(self, active: bool) -> &'static str {
        match (self, active) {
            (Self::Mercury, true) => "bg-mercury",
            (Self::Venus, true) => "bg-venus",
            (Self::Earth, true) => "bg-earth",
            (Self::Mars, true) => "bg-mars",
            (Self::Mercury, false) => "bg-planet hover:bg-mercury",
            (Self::Venus, false) => "bg-planet hover:bg-venus",
            (Self::Earth, false) => "bg-planet hover:bg-earth",
            (Self::Mars, false) => "bg-planet hover:bg-mars",
            _ => "bg-faint-dots",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Cell {
    Empty,
    Faint(Planet),
    Sun { active: bool },
    Planet { planet: Planet, active: bool },
}

fn cells(route: &Route) -> [Cell; 95] {
    let mut grid = [Cell::Empty; 95];
    let home = matches!(route, Route::Home {});
    for row in 1..4 {
        for column in 0..3 {
            grid[row * 19 + column] = Cell::Sun { active: home };
        }
    }

    let planets = [
        (
            4,
            Planet::Mercury,
            matches!(route, Route::Blog {} | Route::BlogPost { .. }),
        ),
        (6, Planet::Venus, matches!(route, Route::Projects {})),
        (8, Planet::Earth, matches!(route, Route::AboutMe {})),
        (10, Planet::Mars, matches!(route, Route::Guestbook {})),
    ];
    for (column, planet, active) in planets {
        let row = if active { 1 } else { 2 };
        grid[row * 19 + column] = Cell::Planet { planet, active };
    }

    for row in 2..5 {
        grid[row * 19 + 12] = Cell::Faint(Planet::Jupiter);
        grid[row * 19 + 14] = Cell::Faint(Planet::Saturn);
    }
    for row in 2..4 {
        grid[row * 19 + 16] = Cell::Faint(Planet::Uranus);
        grid[row * 19 + 18] = Cell::Faint(Planet::Neptune);
    }
    grid
}

fn planet_route(planet: Planet) -> Route {
    match planet {
        Planet::Mercury => Route::Blog {},
        Planet::Venus => Route::Projects {},
        Planet::Earth => Route::AboutMe {},
        Planet::Mars => Route::Guestbook {},
        Planet::Jupiter | Planet::Saturn | Planet::Uranus | Planet::Neptune => Route::Home {},
    }
}

#[component]
pub fn AreciboFooter() -> Element {
    let route: Route = use_route();
    let mut label = use_signal(|| "sol system, from arecibo");
    let year = time::OffsetDateTime::now_utc().year();

    rsx! {
        div { class: "flex flex-col gap-[14px] border-t border-line pt-4",
            div {
                class: "flex w-full items-end gap-2",
                title: "the solar system, roughly",
                div { class: "grid grid-cols-[repeat(19,7px)] grid-rows-[repeat(5,7px)]",
                    for (index, cell) in cells(&route).into_iter().enumerate() {
                        match cell {
                            Cell::Empty => rsx! { span { key: "{index}", class: "block size-[7px]" } },
                            Cell::Sun { active } => rsx! {
                                span {
                                    key: "{index}",
                                    class: "block size-[7px]",
                                    onmouseenter: move |_| label.set("sun"),
                                    onmouseleave: move |_| label.set("sol system, from arecibo"),
                                    Link {
                                        to: Route::Home {},
                                        class: if active { "block size-[7px] bg-sun shadow-[0_0_10px_rgba(236,167,44,.6)]" } else { "block size-[7px] bg-sun" },
                                        title: "sun",
                                        aria_label: "home · sun",
                                    }
                                }
                            },
                            Cell::Planet { planet, active } => rsx! {
                                span {
                                    key: "{index}",
                                    class: "block size-[7px]",
                                    onmouseenter: move |_| label.set(planet.name()),
                                    onmouseleave: move |_| label.set("sol system, from arecibo"),
                                    Link {
                                        to: planet_route(planet),
                                        class: "block size-[7px] {planet.class(active)}",
                                        title: planet.name(),
                                        aria_label: "{planet.name()}",
                                    }
                                }
                            },
                            Cell::Faint(planet) => rsx! {
                                span {
                                    key: "{index}",
                                    class: "block size-[7px] bg-faint-dots",
                                    title: planet.name(),
                                    onmouseenter: move |_| label.set(planet.name()),
                                    onmouseleave: move |_| label.set("sol system, from arecibo"),
                                }
                            },
                        }
                    }
                }
                img {
                    class: "mb-[7px] size-[11px] opacity-85",
                    src: asset!("/assets/heart.svg"),
                    alt: "Pluto",
                    title: "With love, Pluto",
                }
                div {
                    class: "mb-0.5 ml-auto grid grid-cols-6 gap-[3px]",
                    title: "{year}",
                    for shift in (0..12).rev() {
                        span {
                            key: "{shift}",
                            class: if year & (1 << shift) != 0 { "block size-[7px] bg-text" } else { "block size-[7px] bg-empty-cell" },
                        }
                    }
                }
            }
            span { class: "mono -mt-1.5 text-[11px] text-faint", "{label}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lays_out_sun_planets_and_outer_system() {
        let home = cells(&Route::Home {});
        assert_eq!(
            home.iter()
                .filter(|cell| matches!(cell, Cell::Sun { .. }))
                .count(),
            9
        );
        assert!(matches!(
            home[2 * 19 + 4],
            Cell::Planet {
                planet: Planet::Mercury,
                active: false
            }
        ));

        let blog = cells(&Route::Blog {});
        assert!(matches!(
            blog[19 + 4],
            Cell::Planet {
                planet: Planet::Mercury,
                active: true
            }
        ));
        assert!(matches!(blog[4 * 19 + 12], Cell::Faint(Planet::Jupiter)));
    }
}

use crate::Route;
use dioxus::prelude::*;
#[derive(Props, Clone, Debug, PartialEq, Eq)]
struct AreciboDateProps {
    current_year: i32,
}
impl AreciboDateProps {
    fn year(&self) -> i32 {
        self.current_year
    }
}
fn AreciboDate(date: AreciboDateProps) -> Element {
    let year = date.year();
    let binary_year = format!("{:012b}", year);
    let grid: Vec<bool> = binary_year.chars().map(|c| c == '1').collect();
    rsx! {
        div { class: "grid grid-cols-6 gap-1", title: format!("{year}"),
            {
                grid.iter()
                    .map(|&bit| {
                        rsx! {
                            div { class: if bit { "bg-stone-100" } else { "bg-nasty-black" }, class: "w-2 h-2" }
                        }
                    })
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
struct GridProps<const I: usize, const J: usize> {
    grid: [[GridElement; J]; I],
}
impl<const I: usize, const J: usize> GridProps<I, J> {
    fn builder() -> GridPropsBuilder<I, J> {
        GridPropsBuilder::new()
    }
    #[allow(dead_code)]
    fn at(&self, i: usize, j: usize) -> &GridElement {
        &self.grid[i][j]
    }
}
struct GridPropsBuilder<const I: usize, const J: usize> {
    grid: [[Option<GridElement>; J]; I],
}
impl<const I: usize, const J: usize> GridPropsBuilder<I, J> {
    fn new() -> Self {
        let arr: [[Option<GridElement>; J]; I] =
            core::array::from_fn(|_| core::array::from_fn(|_| None));
        Self { grid: arr }
    }
    fn with(mut self, i: usize, j: usize, element: GridElement) -> Self {
        self.grid[i][j] = Some(element);
        self
    }
    fn with_range(
        mut self,
        i: std::ops::Range<usize>,
        j: std::ops::Range<usize>,
        element: GridElement,
    ) -> Self {
        for i in i {
            for j in j.clone() {
                self.grid[i][j] = Some(element.clone());
            }
        }
        self
    }
    fn build(self) -> GridProps<I, J> {
        let grid = self
            .grid
            .map(|row| row.map(|cell| cell.unwrap_or_default()));
        GridProps { grid }
    }
}
#[derive(Clone, Debug, PartialEq)]
struct GridElement {
    inner: Element,
}
impl GridElement {
    fn new(inner: Element) -> Self {
        Self { inner }
    }
}
impl Default for GridElement {
    fn default() -> Self {
        Self {
            inner: rsx! {
                div { class: "w-2 h-2" }
            },
        }
    }
}
fn Grid<const I: usize, const J: usize>(grid: GridProps<I, J>) -> Element {
    rsx! {
        div {
            class: "grid",
            style: "grid-template-columns: repeat({J}, minmax(0, 1fr)); grid-template-rows: repeat({I}, minmax(0, 1fr));",
            {grid.grid.iter().flat_map(|row| row.iter()).map(|cell| cell.inner.clone())}
        }
    }
}
#[component]
fn AreciboIcons() -> Element {
    let route: Route = use_route();
    let grid = GridProps::<5, 19>::builder()
        .with_range(
            1..4,
            0..3,
            GridElement::new(
                rsx! {
                    Link { to: Route::Home {},
                        div {
                            class: "w-2 h-2 bg-[#eca72c]",
                            style: if matches!(route, Route::Home {}) { "box-shadow: 0 40px 20px #eca72c" } else { "" },
                            title: "Sun",
                        }
                    }
                },
            ),
        )
        .with(
            if matches!(route, Route::Blog {}) { 1 } else { 2 },
            4,
            GridElement::new(
                rsx! {
                    Link { to: Route::Blog {},
                        if matches!(route, Route::Blog {}) {
                            div {
                                class: "w-2 h-2 bg-stone-400",
                                title: "Mercury",
                            }
                        } else {
                            div {
                                class: "w-2 h-2 bg-stone-300 hover:bg-stone-400",
                                title: "Mercury",
                            }
                        }
                    }
                }
            ),
        )
        .with(
            if matches!(route, Route::Projects {}) { 1 } else { 2 },
            6,
            GridElement::new(
                rsx! {
                    Link { to: Route::Projects {},
                        if matches!(route, Route::Projects {}) {
                            div {
                                class: "w-2 h-2 bg-orange-300",
                                title: "Venus",
                            }
                        } else {
                            div {
                                class: "w-2 h-2 bg-stone-300 hover:bg-orange-300",
                                title: "Venus",
                            }
                        }
                    }
                },
            ),
        )
        .with(
            if matches!(route, Route::AboutMe {}) { 1 } else { 2 },
            8,
            GridElement::new(
                rsx! {
                    Link { to: Route::AboutMe {},
                        if matches!(route, Route::AboutMe {}) {
                            div { class: "w-2 h-2 bg-glaucolus", title: "Earth" }
                        } else {
                            div {
                                class: "w-2 h-2 bg-stone-300 hover:bg-glaucolus",
                                title: "Earth",
                            }
                        }
                    }
                },
            ),
        )
        .with(
            if matches!(route, Route::Guestbook {}) { 1 } else { 2 },
            10,
            GridElement::new(
                rsx! {
                    Link { to: Route::Guestbook {},
                        if matches!(route, Route::Guestbook {}) {
                            div { class: "w-2 h-2 bg-coral", title: "Mars" }
                        } else {
                            div {
                                class: "w-2 h-2 bg-stone-300 hover:bg-coral",
                                title: "Mars",
                            }
                        }
                    }
                },
            ),
        )
        .with_range(
            2..5,
            12..13,
            GridElement::new(
                rsx! {
                    div { class: "w-2 h-2", title: "Jupiter" }
                },
            ),
        )
        .with_range(
            2..5,
            14..15,
            GridElement::new(
                rsx! {
                    div { class: "w-2 h-2", title: "Saturn" }
                },
            ),
        )
        .with_range(
            2..4,
            16..17,
            GridElement::new(
                rsx! {
                    div { class: "w-2 h-2", title: "Uranus" }
                },
            ),
        )
        .with_range(
            2..4,
            18..19,
            GridElement::new(
                rsx! {
                    div { class: "w-2 h-2", title: "Neptune" }
                },
            ),
        )
        .build();
    rsx! {
        div { class: "flex items-center gap-2",
            {Grid(grid)}
            img {
                class: "w-4 h-4",
                src: asset!("/public/heart.svg"),
                alt: "Pluto",
                title: "With love, Pluto",
            }
        }
    }
}
#[component]
pub fn Footer() -> Element {
    let year = time::OffsetDateTime::now_utc().year();
    rsx! {
        footer { class: "bg-nasty-black text-stone-100 fixed bottom-0 left-0 right-0 z-10 border-t border-jet py-2",
            div { class: "max-w-screen-xl mx-auto flex justify-between items-center px-4",
                AreciboIcons {}
                AreciboDate { current_year: year }
            }
        }
    }
}

mod home;
pub use home::*;
mod about;
pub use about::*;
mod not_found;
pub use not_found::NotFound;
mod projects;
pub use projects::*;
mod guestbook;
pub use guestbook::*;
mod under_construction;
use dioxus::prelude::*;
pub use under_construction::*;
pub fn Blog() -> Element {
    rsx! {
        UnderConstruction {}
    }
}

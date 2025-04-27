use crate::shared::models::{Guest, GuestbookEntry};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserState {
    pub guest: Guest,
    pub entry: Option<GuestbookEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthState {
    Loading,
    Authenticated(Box<UserState>),
    Unauthenticated,
}

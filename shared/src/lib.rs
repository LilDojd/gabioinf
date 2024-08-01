use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
}

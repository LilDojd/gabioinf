use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, sqlx::Type)]
#[sqlx(type_name = "groupvariant", rename_all = "lowercase")]
pub enum Group {
    Admins,
    Guests,
    NaughtyGuests,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroupEntry {
    pub id: i16,
    pub name: Group,
}

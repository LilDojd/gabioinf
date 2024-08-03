use serde::Deserialize;
/// Represents the different types of user groups in the system.
///
/// This enum is used both in the application logic and as a database type,
/// corresponding to the 'groupvariant' type in the database.
#[derive(Debug, Clone, Deserialize, sqlx::Type)]
#[sqlx(type_name = "groupvariant", rename_all = "lowercase")]
pub enum Group {
    /// Administrators with full system access.
    Admins,
    /// Regular users with standard permissions.
    Guests,
    /// Users who have been flagged for inappropriate behavior.
    NaughtyGuests,
}
/// Represents a group entry as stored in the database.
///
/// This struct combines the group's unique identifier and its type.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupEntry {
    /// The unique identifier for the group.
    pub id: i16,
    /// The type of the group.
    pub name: Group,
}

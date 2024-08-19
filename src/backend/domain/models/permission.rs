use serde::Deserialize;
/// Represents the different types of permissions available in the system.
///
/// This enum is used both in the application logic and as a database type,
/// corresponding to the 'permissionvariant' type in the database.
#[derive(Debug, Clone, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "permissionvariant", rename_all = "lowercase")]
pub enum PermissionTargets {
    /// Permission to add a signature to the guestbook.
    AddSignature,
    /// Permission to delete one's own signature from the guestbook.
    DeleteOwnSignature,
    /// Permission to delete any signature from the guestbook.
    DeleteAnySignature,
    /// Permission to edit one's own signature in the guestbook.
    EditOwnSignature,
    /// Permission to mark a user as naughty.
    MarkAsNaughty,
    /// Permission to delete a user from the system.
    DeleteUser,
    /// Permission to promote or demote a user's status.
    ProDemoteUser,
    /// Permission to edit a user's permissions.
    EditUserPermissions,
}
/// Represents a permission entry as stored in the database.
///
/// This struct combines the permission's unique identifier and its type.
#[derive(Debug, Clone, Deserialize)]
pub struct PermissionEntry {
    /// The unique identifier for the permission.
    #[allow(dead_code)]
    pub id: i16,
    /// The type of the permission.
    pub name: PermissionTargets,
}

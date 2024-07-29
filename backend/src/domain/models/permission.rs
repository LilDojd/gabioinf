use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "permissionvariant", rename_all = "lowercase")]
pub enum PermissionTargets {
    AddSignature,
    DeleteOwnSignature,
    DeleteAnySignature,
    EditOwnSignature,
    MarkAsNaughty,
    DeleteUser,
    ProDemoteUser,
    EditUserPermissions,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionEntry {
    pub id: i16,
    pub name: PermissionTargets,
}

//! This module provides functionality for managing groups and permissions for users.
//!
//! The `GroupsAndPermissionsRepo` struct offers methods to add or remove users from groups,
//! assign or revoke permissions, and retrieve user groups and permissions.
use crate::backend::domain::models::{
    Group, GroupEntry, GuestId, PermissionEntry, PermissionTargets,
};
use crate::backend::errors::BResult;
#[derive(Clone, Debug)]
pub struct GroupsAndPermissionsRepo {
    pool: sqlx::PgPool,
}
impl GroupsAndPermissionsRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
    /// Adds a user to a specified group.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to add to the group.
    /// * `group` - The group to which the user will be added.
    ///
    /// # Returns
    ///
    /// A `BResult<()>` indicating success or failure.
    pub async fn add_user_to_group(
        &self,
        user_id: GuestId,
        group: Group,
    ) -> BResult<()> {
        sqlx::query!(
            "INSERT INTO guests_groups (guest_id, group_id)
             SELECT $1, id FROM groups WHERE name = $2
             ON CONFLICT DO NOTHING",
            user_id.as_value(), group as Group
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    /// Removes a user from a specified group.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to remove from the group.
    /// * `group` - The group from which the user will be removed.
    ///
    /// # Returns
    ///
    /// A `BResult<()>` indicating success or failure.
    pub async fn remove_user_from_group(
        &self,
        user_id: GuestId,
        group: Group,
    ) -> BResult<()> {
        sqlx::query!(
            "DELETE FROM guests_groups
             WHERE guest_id = $1 AND group_id = (SELECT id FROM groups WHERE name = $2)",
            user_id.as_value(), group as Group
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    /// Adds a specific permission to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to whom the permission will be added.
    /// * `permission` - The permission to be added to the user.
    ///
    /// # Returns
    ///
    /// A `BResult<()>` indicating success or failure.
    pub async fn add_permission_to_user(
        &self,
        user_id: GuestId,
        permission: PermissionTargets,
    ) -> BResult<()> {
        sqlx::query!(
            "INSERT INTO guests_permissions (guest_id, permission_id)
             SELECT $1, id FROM permissions WHERE name = $2
             ON CONFLICT DO NOTHING",
            user_id.as_value(), permission as PermissionTargets
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    /// Removes a specific permission from a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user from whom the permission will be removed.
    /// * `permission` - The permission to be removed from the user.
    ///
    /// # Returns
    ///
    /// A `BResult<()>` indicating success or failure.
    pub async fn remove_permission_from_user(
        &self,
        user_id: GuestId,
        permission: PermissionTargets,
    ) -> BResult<()> {
        sqlx::query!(
            "DELETE FROM guests_permissions
             WHERE guest_id = $1 AND permission_id = (SELECT id FROM permissions WHERE name = $2)",
            user_id.as_value(), permission as PermissionTargets
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    /// Retrieves all groups a user belongs to.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user whose groups are to be retrieved.
    ///
    /// # Returns
    ///
    /// A `BResult<Vec<Group>>` containing the groups the user belongs to.
    pub async fn get_user_groups(&self, user_id: GuestId) -> BResult<Vec<Group>> {
        let groups = sqlx::query_as!(
            GroupEntry,
            r#"SELECT g.id, g.name as "name: _"
             FROM groups g
             JOIN guests_groups gg ON g.id = gg.group_id
             WHERE gg.guest_id = $1"#,
            user_id.as_value()
        )
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|g| g.name.clone())
            .collect();
        Ok(groups)
    }
    /// Retrieves permissions specifically assigned to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user whose specific permissions are to be retrieved.
    ///
    /// # Returns
    ///
    /// A `BResult<Vec<PermissionTargets>>` containing the user's specific permissions.
    pub async fn get_user_specific_permissions(
        &self,
        user_id: GuestId,
    ) -> BResult<Vec<PermissionTargets>> {
        let permissions = sqlx::query_as!(
            PermissionEntry,
            r#"SELECT p.id, p.name as "name: PermissionTargets"
             FROM permissions p
             JOIN guests_permissions gp ON p.id = gp.permission_id
             WHERE gp.guest_id = $1"#,
            user_id.as_value()
        )
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|p| p.name.clone())
            .collect();
        Ok(permissions)
    }
    /// Retrieves permissions a user has through their group memberships.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user whose group permissions are to be retrieved.
    ///
    /// # Returns
    ///
    /// A `BResult<Vec<PermissionTargets>>` containing the user's group-based permissions.
    pub async fn get_user_group_permissions(
        &self,
        user_id: GuestId,
    ) -> BResult<Vec<PermissionTargets>> {
        let permissions = sqlx::query_as!(
            PermissionEntry,
            r#"SELECT DISTINCT p.id, p.name as "name: PermissionTargets"
             FROM permissions p
             JOIN groups_permissions grp ON p.id = grp.permission_id
             JOIN guests_groups gg ON grp.group_id = gg.group_id
             WHERE gg.guest_id = $1"#,
            user_id.as_value()
        )
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|p| p.name.clone())
            .collect();
        Ok(permissions)
    }
    /// Retrieves all permissions a user has, both specific and group-based.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user whose permissions are to be retrieved.
    ///
    /// # Returns
    ///
    /// A `BResult<Vec<PermissionTargets>>` containing all of the user's permissions.
    pub async fn get_all_user_permissions(
        &self,
        user_id: GuestId,
    ) -> BResult<Vec<PermissionTargets>> {
        let permissions = sqlx::query_as!(
            PermissionEntry,
            r#"SELECT DISTINCT p.id, p.name as "name: PermissionTargets"
             FROM permissions p
             LEFT JOIN guests_permissions gp ON p.id = gp.permission_id
             LEFT JOIN groups_permissions grp ON p.id = grp.permission_id
             LEFT JOIN guests_groups gg ON grp.group_id = gg.group_id
             WHERE gp.guest_id = $1 OR gg.guest_id = $1"#,
            user_id.as_value()
        )
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|p| p.name.clone())
            .collect();
        Ok(permissions)
    }
}

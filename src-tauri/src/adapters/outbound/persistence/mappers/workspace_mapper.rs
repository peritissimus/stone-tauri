//! Workspace Entity Mapper
//!
//! Maps between database schema and domain Workspace entity.

use diesel::prelude::*;
use crate::domain::entities::Workspace;
use crate::shared::database::schema::workspaces;
use super::super::utils::{
    datetime_to_timestamp, timestamp_to_datetime, bool_to_i32, i32_to_bool,
};

/// Database row struct for workspaces table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = workspaces)]
pub struct WorkspaceRow {
    pub id: String,
    pub name: String,
    pub folder_path: String,
    pub is_active: i32,
    pub created_at: i64,
    pub last_accessed_at: i64,
}

/// Insertable struct for workspaces table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = workspaces)]
pub struct InsertableWorkspace {
    pub id: String,
    pub name: String,
    pub folder_path: String,
    pub is_active: i32,
    pub created_at: i64,
    pub last_accessed_at: i64,
}

impl WorkspaceRow {
    /// Convert database row to domain entity
    pub fn to_domain(self) -> Workspace {
        Workspace {
            id: self.id,
            name: self.name,
            folder_path: self.folder_path,
            is_active: i32_to_bool(self.is_active),
            created_at: timestamp_to_datetime(self.created_at),
            last_accessed_at: timestamp_to_datetime(self.last_accessed_at),
        }
    }
}

impl InsertableWorkspace {
    /// Convert domain entity to insertable struct
    pub fn from_domain(workspace: &Workspace) -> Self {
        Self {
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            folder_path: workspace.folder_path.clone(),
            is_active: bool_to_i32(workspace.is_active),
            created_at: datetime_to_timestamp(&workspace.created_at),
            last_accessed_at: datetime_to_timestamp(&workspace.last_accessed_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_workspace_mapping() {
        let workspace = Workspace {
            id: "ws1".to_string(),
            name: "My Workspace".to_string(),
            folder_path: "/path/to/workspace".to_string(),
            is_active: true,
            created_at: Utc::now(),
            last_accessed_at: Utc::now(),
        };

        let insertable = InsertableWorkspace::from_domain(&workspace);
        assert_eq!(insertable.id, workspace.id);
        assert_eq!(insertable.name, workspace.name);
        assert_eq!(insertable.is_active, 1);

        let row = WorkspaceRow {
            id: insertable.id.clone(),
            name: insertable.name.clone(),
            folder_path: insertable.folder_path.clone(),
            is_active: insertable.is_active,
            created_at: insertable.created_at,
            last_accessed_at: insertable.last_accessed_at,
        };

        let domain = row.to_domain();
        assert_eq!(domain.id, workspace.id);
        assert_eq!(domain.name, workspace.name);
        assert_eq!(domain.is_active, workspace.is_active);
    }
}

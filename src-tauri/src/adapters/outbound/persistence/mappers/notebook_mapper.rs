//! Notebook Entity Mapper
//!
//! Maps between database schema and domain Notebook entity.

use diesel::prelude::*;
use crate::domain::entities::Notebook;
use crate::shared::database::schema::notebooks;
use super::super::utils::{datetime_to_timestamp, timestamp_to_datetime};

/// Database row struct for notebooks table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = notebooks)]
pub struct NotebookRow {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub folder_path: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub position: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Insertable struct for notebooks table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = notebooks)]
pub struct InsertableNotebook {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub folder_path: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub position: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl NotebookRow {
    /// Convert database row to domain entity
    pub fn to_domain(self) -> Notebook {
        Notebook {
            id: self.id,
            name: self.name,
            parent_id: self.parent_id,
            workspace_id: self.workspace_id,
            folder_path: self.folder_path,
            icon: self.icon.unwrap_or_else(|| "📁".to_string()),
            color: self.color.unwrap_or_else(|| "#3b82f6".to_string()),
            position: self.position.unwrap_or(0),
            created_at: timestamp_to_datetime(self.created_at),
            updated_at: timestamp_to_datetime(self.updated_at),
        }
    }
}

impl InsertableNotebook {
    /// Convert domain entity to insertable struct
    pub fn from_domain(notebook: &Notebook) -> Self {
        Self {
            id: notebook.id.clone(),
            name: notebook.name.clone(),
            parent_id: notebook.parent_id.clone(),
            workspace_id: notebook.workspace_id.clone(),
            folder_path: notebook.folder_path.clone(),
            icon: Some(notebook.icon.clone()),
            color: Some(notebook.color.clone()),
            position: Some(notebook.position),
            created_at: datetime_to_timestamp(&notebook.created_at),
            updated_at: datetime_to_timestamp(&notebook.updated_at),
        }
    }
}

//! Note Entity Mapper
//!
//! Maps between database schema and domain Note entity.

use diesel::prelude::*;
use crate::domain::entities::Note;
use crate::shared::database::schema::notes;
use super::super::utils::{
    datetime_to_timestamp, timestamp_to_datetime, optional_datetime_to_timestamp,
    optional_timestamp_to_datetime, serialize_embedding, deserialize_embedding,
    bool_to_i32, i32_to_bool,
};

/// Database row struct for notes table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = notes)]
pub struct NoteRow {
    pub id: String,
    pub title: Option<String>,
    pub file_path: Option<String>,
    pub notebook_id: Option<String>,
    pub workspace_id: Option<String>,
    pub is_favorite: i32,
    pub is_pinned: i32,
    pub is_archived: i32,
    pub is_deleted: i32,
    pub deleted_at: Option<i64>,
    pub embedding: Option<Vec<u8>>, // Not mapped to domain - handled separately
    pub created_at: i64,
    pub updated_at: i64,
}

/// Insertable struct for notes table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = notes)]
pub struct InsertableNote {
    pub id: String,
    pub title: Option<String>,
    pub file_path: Option<String>,
    pub notebook_id: Option<String>,
    pub workspace_id: Option<String>,
    pub is_favorite: i32,
    pub is_pinned: i32,
    pub is_archived: i32,
    pub is_deleted: i32,
    pub deleted_at: Option<i64>,
    // embedding field omitted - handled separately via update methods
    pub created_at: i64,
    pub updated_at: i64,
}

impl NoteRow {
    /// Convert database row to domain entity
    /// Note: embedding field is not mapped - it's handled separately
    pub fn to_domain(self) -> Note {
        Note {
            id: self.id,
            title: self.title.unwrap_or_default(),
            file_path: self.file_path,
            notebook_id: self.notebook_id,
            workspace_id: self.workspace_id,
            is_favorite: i32_to_bool(self.is_favorite),
            is_pinned: i32_to_bool(self.is_pinned),
            is_archived: i32_to_bool(self.is_archived),
            is_deleted: i32_to_bool(self.is_deleted),
            deleted_at: optional_timestamp_to_datetime(self.deleted_at),
            created_at: timestamp_to_datetime(self.created_at),
            updated_at: timestamp_to_datetime(self.updated_at),
        }
    }
}

impl InsertableNote {
    /// Convert domain entity to insertable struct
    /// Note: embedding field is not included - it's handled separately
    pub fn from_domain(note: &Note) -> Self {
        Self {
            id: note.id.clone(),
            title: if note.title.is_empty() {
                None
            } else {
                Some(note.title.clone())
            },
            file_path: note.file_path.clone(),
            notebook_id: note.notebook_id.clone(),
            workspace_id: note.workspace_id.clone(),
            is_favorite: bool_to_i32(note.is_favorite),
            is_pinned: bool_to_i32(note.is_pinned),
            is_archived: bool_to_i32(note.is_archived),
            is_deleted: bool_to_i32(note.is_deleted),
            deleted_at: optional_datetime_to_timestamp(&note.deleted_at),
            created_at: datetime_to_timestamp(&note.created_at),
            updated_at: datetime_to_timestamp(&note.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_note_mapping() {
        let note = Note {
            id: "note1".to_string(),
            title: "Test Note".to_string(),
            file_path: Some("notes/test.md".to_string()),
            notebook_id: Some("nb1".to_string()),
            workspace_id: Some("ws1".to_string()),
            is_favorite: true,
            is_pinned: false,
            is_archived: false,
            is_deleted: false,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let insertable = InsertableNote::from_domain(&note);
        assert_eq!(insertable.id, note.id);
        assert_eq!(insertable.title, Some(note.title.clone()));
        assert_eq!(insertable.is_favorite, 1);

        let row = NoteRow {
            id: insertable.id.clone(),
            title: insertable.title.clone(),
            file_path: insertable.file_path.clone(),
            notebook_id: insertable.notebook_id.clone(),
            workspace_id: insertable.workspace_id.clone(),
            is_favorite: insertable.is_favorite,
            is_pinned: insertable.is_pinned,
            is_archived: insertable.is_archived,
            is_deleted: insertable.is_deleted,
            deleted_at: insertable.deleted_at,
            embedding: None, // Not mapped
            created_at: insertable.created_at,
            updated_at: insertable.updated_at,
        };

        let domain = row.to_domain();
        assert_eq!(domain.id, note.id);
        assert_eq!(domain.title, note.title);
        assert_eq!(domain.is_favorite, note.is_favorite);
    }

    #[test]
    fn test_note_mapping_empty_title() {
        let note = Note {
            id: "note2".to_string(),
            title: "".to_string(),
            file_path: None,
            notebook_id: None,
            workspace_id: None,
            is_favorite: false,
            is_pinned: false,
            is_archived: false,
            is_deleted: false,
            deleted_at: None,
            embedding: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let insertable = InsertableNote::from_domain(&note);
        assert_eq!(insertable.title, None); // Empty title becomes None
    }
}

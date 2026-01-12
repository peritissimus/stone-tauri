//! Note Link Entity Mapper
//!
//! Maps between database schema and domain NoteLink entity.

use diesel::prelude::*;
use crate::domain::entities::NoteLink;
use crate::shared::database::schema::note_links;
use super::super::utils::{datetime_to_timestamp, timestamp_to_datetime};

/// Database row struct for note_links table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = note_links)]
pub struct NoteLinkRow {
    pub source_note_id: String,
    pub target_note_id: String,
    pub created_at: i64,
}

/// Insertable struct for note_links table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = note_links)]
pub struct InsertableNoteLink {
    pub source_note_id: String,
    pub target_note_id: String,
    pub created_at: i64,
}

impl NoteLinkRow {
    /// Convert database row to domain entity
    pub fn to_domain(self) -> NoteLink {
        NoteLink {
            source_note_id: self.source_note_id,
            target_note_id: self.target_note_id,
            created_at: timestamp_to_datetime(self.created_at),
        }
    }
}

impl InsertableNoteLink {
    /// Convert domain entity to insertable struct
    pub fn from_domain(link: &NoteLink) -> Self {
        Self {
            source_note_id: link.source_note_id.clone(),
            target_note_id: link.target_note_id.clone(),
            created_at: datetime_to_timestamp(&link.created_at),
        }
    }
}

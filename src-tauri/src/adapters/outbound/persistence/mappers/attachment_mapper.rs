//! Attachment Entity Mapper
//!
//! Maps between database schema and domain Attachment entity.

use diesel::prelude::*;
use crate::domain::entities::Attachment;
use crate::shared::database::schema::attachments;
use super::super::utils::{datetime_to_timestamp, timestamp_to_datetime};

/// Database row struct for attachments table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = attachments)]
pub struct AttachmentRow {
    pub id: String,
    pub note_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i32,
    pub path: String,
    pub created_at: i64,
}

/// Insertable struct for attachments table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = attachments)]
pub struct InsertableAttachment {
    pub id: String,
    pub note_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i32,
    pub path: String,
    pub created_at: i64,
}

impl AttachmentRow {
    /// Convert database row to domain entity
    pub fn to_domain(self) -> Attachment {
        Attachment {
            id: self.id,
            note_id: self.note_id,
            filename: self.filename,
            mime_type: self.mime_type,
            size: self.size as i64,
            path: self.path,
            created_at: timestamp_to_datetime(self.created_at),
        }
    }
}

impl InsertableAttachment {
    /// Convert domain entity to insertable struct
    pub fn from_domain(attachment: &Attachment) -> Self {
        Self {
            id: attachment.id.clone(),
            note_id: attachment.note_id.clone(),
            filename: attachment.filename.clone(),
            mime_type: attachment.mime_type.clone(),
            size: attachment.size as i32, // Truncate to i32 for SQLite INTEGER
            path: attachment.path.clone(),
            created_at: datetime_to_timestamp(&attachment.created_at),
        }
    }
}

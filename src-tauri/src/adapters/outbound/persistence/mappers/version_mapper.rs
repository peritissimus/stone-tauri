//! Note Version Entity Mapper
//!
//! Maps between database schema and domain Version entity.

use diesel::prelude::*;
use crate::domain::entities::Version;
use crate::shared::database::schema::note_versions;
use super::super::utils::{datetime_to_timestamp, timestamp_to_datetime};

/// Database row struct for note_versions table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = note_versions)]
pub struct VersionRow {
    pub id: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub version_number: i32,
    pub created_at: i64,
}

/// Insertable struct for note_versions table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = note_versions)]
pub struct InsertableVersion {
    pub id: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub version_number: i32,
    pub created_at: i64,
}

impl VersionRow {
    /// Convert database row to domain entity
    pub fn to_domain(self) -> Version {
        Version {
            id: self.id,
            note_id: self.note_id,
            title: self.title,
            content: self.content,
            version_number: self.version_number,
            created_at: timestamp_to_datetime(self.created_at),
        }
    }
}

impl InsertableVersion {
    /// Convert domain entity to insertable struct
    pub fn from_domain(version: &Version) -> Self {
        Self {
            id: version.id.clone(),
            note_id: version.note_id.clone(),
            title: version.title.clone(),
            content: version.content.clone(),
            version_number: version.version_number,
            created_at: datetime_to_timestamp(&version.created_at),
        }
    }
}

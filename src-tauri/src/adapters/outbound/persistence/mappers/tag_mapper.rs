//! Tag Entity Mapper
//!
//! Maps between database schema and domain Tag entity.

use diesel::prelude::*;
use crate::domain::entities::Tag;
use crate::shared::database::schema::tags;
use super::super::utils::{datetime_to_timestamp, timestamp_to_datetime};

/// Database row struct for tags table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = tags)]
pub struct TagRow {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Insertable struct for tags table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = tags)]
pub struct InsertableTag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TagRow {
    /// Convert database row to domain entity
    pub fn to_domain(self) -> Tag {
        Tag {
            id: self.id,
            name: self.name,
            color: self.color.unwrap_or_else(|| "#6b7280".to_string()),
            created_at: timestamp_to_datetime(self.created_at),
            updated_at: timestamp_to_datetime(self.updated_at),
        }
    }
}

impl InsertableTag {
    /// Convert domain entity to insertable struct
    pub fn from_domain(tag: &Tag) -> Self {
        Self {
            id: tag.id.clone(),
            name: tag.name.clone(),
            color: Some(tag.color.clone()),
            created_at: datetime_to_timestamp(&tag.created_at),
            updated_at: datetime_to_timestamp(&tag.updated_at),
        }
    }
}

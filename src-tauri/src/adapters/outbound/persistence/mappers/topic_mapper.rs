//! Topic Entity Mapper
//!
//! Maps between database schema and domain Topic entity.

use diesel::prelude::*;
use crate::domain::entities::Topic;
use crate::shared::database::schema::topics;
use super::super::utils::{
    datetime_to_timestamp, timestamp_to_datetime, bool_to_i32, i32_to_bool,
};

/// Database row struct for topics table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = topics)]
pub struct TopicRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_predefined: i32,
    pub centroid: Option<Vec<u8>>,
    pub note_count: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Insertable struct for topics table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = topics)]
pub struct InsertableTopic {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_predefined: i32,
    pub centroid: Option<Vec<u8>>,
    pub note_count: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TopicRow {
    /// Convert database row to domain entity
    pub fn to_domain(self) -> Topic {
        Topic {
            id: self.id,
            name: self.name,
            description: self.description,
            color: self.color.unwrap_or_else(|| Topic::default_color()),
            is_predefined: i32_to_bool(self.is_predefined),
            centroid: self.centroid, // Already stored as Vec<u8>
            note_count: self.note_count.unwrap_or(0),
            created_at: timestamp_to_datetime(self.created_at),
            updated_at: timestamp_to_datetime(self.updated_at),
        }
    }
}

impl InsertableTopic {
    /// Convert domain entity to insertable struct
    pub fn from_domain(topic: &Topic) -> Self {
        Self {
            id: topic.id.clone(),
            name: topic.name.clone(),
            description: topic.description.clone(),
            color: Some(topic.color.clone()),
            is_predefined: bool_to_i32(topic.is_predefined),
            centroid: topic.centroid.clone(), // Already stored as Vec<u8>
            note_count: Some(topic.note_count),
            created_at: datetime_to_timestamp(&topic.created_at),
            updated_at: datetime_to_timestamp(&topic.updated_at),
        }
    }
}

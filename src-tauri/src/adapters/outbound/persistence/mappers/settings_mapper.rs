//! Settings Entity Mapper
//!
//! Maps between database schema and Setting struct from repository port.

use diesel::prelude::*;
use crate::domain::ports::outbound::Setting;
use crate::shared::database::schema::settings;
use super::super::utils::{datetime_to_timestamp, timestamp_to_datetime};

/// Database row struct for settings table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = settings)]
pub struct SettingRow {
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

/// Insertable struct for settings table
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = settings)]
pub struct InsertableSetting {
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

impl SettingRow {
    /// Convert database row to domain entity
    pub fn to_domain(self) -> Setting {
        Setting {
            key: self.key,
            value: self.value,
            updated_at: timestamp_to_datetime(self.updated_at),
        }
    }
}

impl InsertableSetting {
    /// Convert domain entity to insertable struct
    pub fn from_domain(setting: &Setting) -> Self {
        Self {
            key: setting.key.clone(),
            value: setting.value.clone(),
            updated_at: datetime_to_timestamp(&setting.updated_at),
        }
    }
}

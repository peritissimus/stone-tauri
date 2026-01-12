//! Settings Repository Implementation
//!
//! Diesel-based implementation of the Settings repository port (key-value store).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::outbound::{Setting, SettingsRepository},
};
use crate::shared::database::schema::settings;

use super::{
    db_pool::{get_connection, DbPool},
    mappers::{InsertableSetting, SettingRow},
    utils::{datetime_to_timestamp, map_diesel_error},
};

/// Diesel implementation of SettingsRepository
pub struct DieselSettingsRepository {
    pool: Arc<DbPool>,
}

impl DieselSettingsRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsRepository for DieselSettingsRepository {
    /// Get a setting by key
    async fn get(&self, key: &str) -> DomainResult<Option<Setting>> {
        let pool = self.pool.clone();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            settings::table
                .filter(settings::key.eq(key))
                .first::<SettingRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Set a setting (create or update)
    async fn set(&self, key: &str, value: &str) -> DomainResult<Setting> {
        let pool = self.pool.clone();
        let key = key.to_string();
        let value = value.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let setting = Setting {
                key: key.clone(),
                value: value.clone(),
                updated_at: Utc::now(),
            };

            let insertable = InsertableSetting::from_domain(&setting);

            diesel::insert_into(settings::table)
                .values(&insertable)
                .on_conflict(settings::key)
                .do_update()
                .set(&insertable)
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(setting)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get all settings
    async fn get_all(&self) -> DomainResult<Vec<Setting>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = settings::table
                .order(settings::key.asc())
                .load::<SettingRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete a setting by key
    async fn delete(&self, key: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let deleted = diesel::delete(settings::table.filter(settings::key.eq(&key)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            if deleted == 0 {
                return Err(DomainError::NotFound(format!("Setting not found: {}", key)));
            }

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::db_pool::create_pool;

    #[tokio::test]
    async fn test_set_and_get_setting() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselSettingsRepository::new(pool);

        let setting = repo.set("theme", "dark").await.unwrap();
        assert_eq!(setting.key, "theme");
        assert_eq!(setting.value, "dark");

        let retrieved = repo.get("theme").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "dark");
    }

    #[tokio::test]
    async fn test_update_setting() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselSettingsRepository::new(pool);

        // Set initial value
        repo.set("theme", "light").await.unwrap();

        let setting = repo.get("theme").await.unwrap();
        assert!(setting.is_some());
        assert_eq!(setting.unwrap().value, "light");

        // Update value
        repo.set("theme", "dark").await.unwrap();

        let updated = repo.get("theme").await.unwrap();
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().value, "dark");
    }

    #[tokio::test]
    async fn test_get_all_settings() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselSettingsRepository::new(pool);

        repo.set("key1", "value1").await.unwrap();
        repo.set("key2", "value2").await.unwrap();
        repo.set("key3", "value3").await.unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_setting() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselSettingsRepository::new(pool);

        repo.set("test_key", "test_value").await.unwrap();
        assert!(repo.get("test_key").await.unwrap().is_some());

        repo.delete("test_key").await.unwrap();

        let result = repo.get("test_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_update_existing_setting() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselSettingsRepository::new(pool);

        repo.set("key1", "value1").await.unwrap();
        repo.set("key1", "value2").await.unwrap();

        let setting = repo.get("key1").await.unwrap().unwrap();
        assert_eq!(setting.value, "value2");

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 1); // Should still only have 1 entry
    }
}

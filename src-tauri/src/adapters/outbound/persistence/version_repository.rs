//! Version Repository Implementation
//!
//! Diesel-based implementation of the Version repository port for note version history.

use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;

use crate::domain::{
    entities::Version,
    errors::{DomainError, DomainResult},
    ports::outbound::{VersionListItem, VersionRepository},
};
use crate::shared::database::schema::note_versions;

use super::{
    db_pool::{get_connection, DbPool},
    mappers::{InsertableVersion, VersionRow},
    utils::map_diesel_error,
};

/// Diesel implementation of VersionRepository
pub struct DieselVersionRepository {
    pool: Arc<DbPool>,
}

impl DieselVersionRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VersionRepository for DieselVersionRepository {
    /// Find version by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Version>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            note_versions::table
                .filter(note_versions::id.eq(id))
                .first::<VersionRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Create a new version snapshot
    async fn save(&self, version: &Version) -> DomainResult<()> {
        let pool = self.pool.clone();
        let version = version.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let insertable = InsertableVersion::from_domain(&version);

            diesel::insert_into(note_versions::table)
                .values(&insertable)
                .on_conflict(note_versions::id)
                .do_update()
                .set(&insertable)
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get next version number for a note
    async fn get_next_version_number(&self, note_id: &str) -> DomainResult<i32> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let max_version: Option<i32> = note_versions::table
                .filter(note_versions::note_id.eq(note_id))
                .select(diesel::dsl::max(note_versions::version_number))
                .first(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .flatten();

            Ok(max_version.unwrap_or(0) + 1)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get all versions for a note
    async fn find_by_note_id(&self, note_id: &str) -> DomainResult<Vec<Version>> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = note_versions::table
                .filter(note_versions::note_id.eq(note_id))
                .order(note_versions::version_number.desc())
                .load::<VersionRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get version history summary (without full content)
    async fn get_version_summary(&self, note_id: &str) -> DomainResult<Vec<VersionListItem>> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows: Vec<(String, String, i32, String, i64)> = note_versions::table
                .filter(note_versions::note_id.eq(note_id))
                .select((
                    note_versions::id,
                    note_versions::note_id,
                    note_versions::version_number,
                    note_versions::title,
                    note_versions::created_at,
                ))
                .order(note_versions::version_number.desc())
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows
                .into_iter()
                .map(|(id, note_id, version_number, title, created_at)| VersionListItem {
                    id,
                    note_id,
                    version_number,
                    title,
                    created_at: super::utils::timestamp_to_datetime(created_at),
                })
                .collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get latest version for a note
    async fn get_latest_version(&self, note_id: &str) -> DomainResult<Option<Version>> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            note_versions::table
                .filter(note_versions::note_id.eq(note_id))
                .order(note_versions::version_number.desc())
                .first::<VersionRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete all versions for a note
    async fn delete_by_note_id(&self, note_id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            diesel::delete(note_versions::table.filter(note_versions::note_id.eq(note_id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete old versions (keep N most recent)
    async fn prune_versions(&self, note_id: &str, keep_count: i32) -> DomainResult<i32> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Get version numbers to keep (most recent N)
            let versions_to_keep: Vec<i32> = note_versions::table
                .filter(note_versions::note_id.eq(&note_id))
                .select(note_versions::version_number)
                .order(note_versions::version_number.desc())
                .limit(keep_count as i64)
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            if versions_to_keep.is_empty() {
                return Ok(0);
            }

            // Delete versions not in the keep list
            let deleted = diesel::delete(
                note_versions::table
                    .filter(note_versions::note_id.eq(&note_id))
                    .filter(note_versions::version_number.ne_all(versions_to_keep)),
            )
            .execute(&mut conn)
            .map_err(map_diesel_error)?;

            Ok(deleted as i32)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Count versions for a note
    async fn count_by_note_id(&self, note_id: &str) -> DomainResult<i32> {
        let pool = self.pool.clone();
        let note_id = note_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = note_versions::table
                .filter(note_versions::note_id.eq(note_id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count as i32)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::db_pool::create_pool;
    use chrono::Utc;

    fn create_test_version(
        id: &str,
        note_id: &str,
        title: &str,
        version_number: i32,
    ) -> Version {
        Version {
            id: id.to_string(),
            note_id: note_id.to_string(),
            title: title.to_string(),
            content: format!("Content for version {}", version_number),
            version_number,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_version() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselVersionRepository::new(pool);

        let version = create_test_version("v1", "note1", "Test Note", 1);
        repo.save(&version).await.unwrap();

        let found = repo.find_by_id("v1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Note");
    }

    #[tokio::test]
    async fn test_get_next_version_number() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselVersionRepository::new(pool);

        // First version should be 1
        let next = repo.get_next_version_number("note1").await.unwrap();
        assert_eq!(next, 1);

        // Save version 1
        let v1 = create_test_version("v1", "note1", "Test", 1);
        repo.save(&v1).await.unwrap();

        // Next should be 2
        let next = repo.get_next_version_number("note1").await.unwrap();
        assert_eq!(next, 2);

        // Save version 2
        let v2 = create_test_version("v2", "note1", "Test", 2);
        repo.save(&v2).await.unwrap();

        // Next should be 3
        let next = repo.get_next_version_number("note1").await.unwrap();
        assert_eq!(next, 3);
    }

    #[tokio::test]
    async fn test_find_by_note_id() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselVersionRepository::new(pool);

        let v1 = create_test_version("v1", "note1", "Version 1", 1);
        let v2 = create_test_version("v2", "note1", "Version 2", 2);
        let v3 = create_test_version("v3", "note1", "Version 3", 3);

        repo.save(&v1).await.unwrap();
        repo.save(&v2).await.unwrap();
        repo.save(&v3).await.unwrap();

        let versions = repo.find_by_note_id("note1").await.unwrap();
        assert_eq!(versions.len(), 3);

        // Should be in descending order
        assert_eq!(versions[0].version_number, 3);
        assert_eq!(versions[1].version_number, 2);
        assert_eq!(versions[2].version_number, 1);
    }

    #[tokio::test]
    async fn test_get_version_summary() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselVersionRepository::new(pool);

        let v1 = create_test_version("v1", "note1", "Version 1", 1);
        let v2 = create_test_version("v2", "note1", "Version 2", 2);

        repo.save(&v1).await.unwrap();
        repo.save(&v2).await.unwrap();

        let summary = repo.get_version_summary("note1").await.unwrap();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].version_number, 2);
        assert_eq!(summary[1].version_number, 1);
    }

    #[tokio::test]
    async fn test_get_latest_version() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselVersionRepository::new(pool);

        let v1 = create_test_version("v1", "note1", "Version 1", 1);
        let v2 = create_test_version("v2", "note1", "Version 2", 2);
        let v3 = create_test_version("v3", "note1", "Version 3", 3);

        repo.save(&v1).await.unwrap();
        repo.save(&v2).await.unwrap();
        repo.save(&v3).await.unwrap();

        let latest = repo.get_latest_version("note1").await.unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version_number, 3);
    }

    #[tokio::test]
    async fn test_delete_by_note_id() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselVersionRepository::new(pool);

        let v1 = create_test_version("v1", "note1", "Version 1", 1);
        let v2 = create_test_version("v2", "note1", "Version 2", 2);

        repo.save(&v1).await.unwrap();
        repo.save(&v2).await.unwrap();

        repo.delete_by_note_id("note1").await.unwrap();

        let versions = repo.find_by_note_id("note1").await.unwrap();
        assert_eq!(versions.len(), 0);
    }

    #[tokio::test]
    async fn test_prune_versions() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselVersionRepository::new(pool);

        // Create 5 versions
        for i in 1..=5 {
            let v = create_test_version(&format!("v{}", i), "note1", &format!("V{}", i), i);
            repo.save(&v).await.unwrap();
        }

        // Keep only 2 most recent
        let deleted = repo.prune_versions("note1", 2).await.unwrap();
        assert_eq!(deleted, 3); // Should delete 3 old versions

        let versions = repo.find_by_note_id("note1").await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version_number, 5);
        assert_eq!(versions[1].version_number, 4);
    }

    #[tokio::test]
    async fn test_count_by_note_id() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselVersionRepository::new(pool);

        let v1 = create_test_version("v1", "note1", "Version 1", 1);
        let v2 = create_test_version("v2", "note1", "Version 2", 2);
        let v3 = create_test_version("v3", "note1", "Version 3", 3);

        repo.save(&v1).await.unwrap();
        repo.save(&v2).await.unwrap();
        repo.save(&v3).await.unwrap();

        let count = repo.count_by_note_id("note1").await.unwrap();
        assert_eq!(count, 3);
    }
}

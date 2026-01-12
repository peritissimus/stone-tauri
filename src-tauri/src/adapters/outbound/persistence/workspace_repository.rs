//! Workspace Repository Implementation
//!
//! Diesel-based implementation of the Workspace repository port.

use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;

use crate::domain::{
    entities::Workspace,
    errors::{DomainError, DomainResult},
    ports::outbound::WorkspaceRepository,
};
use crate::shared::database::schema::workspaces;

use super::{
    db_pool::{DbPool, get_connection},
    mappers::{InsertableWorkspace, WorkspaceRow},
    utils::map_diesel_error,
};

/// Diesel implementation of WorkspaceRepository
pub struct DieselWorkspaceRepository {
    pool: Arc<DbPool>,
}

impl DieselWorkspaceRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkspaceRepository for DieselWorkspaceRepository {
    /// Find workspace by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Workspace>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            workspaces::table
                .filter(workspaces::id.eq(id))
                .first::<WorkspaceRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find workspace by folder path
    async fn find_by_folder_path(&self, path: &str) -> DomainResult<Option<Workspace>> {
        let pool = self.pool.clone();
        let path = path.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            workspaces::table
                .filter(workspaces::folder_path.eq(path))
                .first::<WorkspaceRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find all workspaces
    async fn find_all(&self) -> DomainResult<Vec<Workspace>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = workspaces::table
                .order(workspaces::name.asc())
                .load::<WorkspaceRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find active workspace
    async fn find_active(&self) -> DomainResult<Option<Workspace>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            workspaces::table
                .filter(workspaces::is_active.eq(1))
                .first::<WorkspaceRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Save workspace (insert or update)
    async fn save(&self, workspace: &Workspace) -> DomainResult<()> {
        let pool = self.pool.clone();
        let workspace = workspace.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let insertable = InsertableWorkspace::from_domain(&workspace);

            diesel::insert_into(workspaces::table)
                .values(&insertable)
                .on_conflict(workspaces::id)
                .do_update()
                .set(&insertable)
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete workspace by ID
    async fn delete(&self, id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let deleted = diesel::delete(workspaces::table.filter(workspaces::id.eq(&id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            if deleted == 0 {
                return Err(DomainError::WorkspaceNotFound(id));
            }

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Set workspace as active (and deactivate others)
    async fn set_active(&self, id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Use transaction to ensure atomicity
            conn.transaction(|conn| {
                // First, deactivate all workspaces
                diesel::update(workspaces::table)
                    .set(workspaces::is_active.eq(0))
                    .execute(conn)?;

                // Then, activate the specified workspace
                let updated = diesel::update(workspaces::table.filter(workspaces::id.eq(&id)))
                    .set(workspaces::is_active.eq(1))
                    .execute(conn)?;

                if updated == 0 {
                    return Err(diesel::result::Error::NotFound);
                }

                Ok(())
            })
            .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Check if workspace exists
    async fn exists(&self, id: &str) -> DomainResult<bool> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = workspaces::table
                .filter(workspaces::id.eq(id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count > 0)
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

    fn create_test_workspace(id: &str, name: &str) -> Workspace {
        Workspace {
            id: id.to_string(),
            name: name.to_string(),
            folder_path: format!("/tmp/{}", id),
            is_active: false,
            created_at: Utc::now(),
            last_accessed_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_workspace() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselWorkspaceRepository::new(pool);

        let workspace = create_test_workspace("ws1", "Test Workspace");
        repo.save(&workspace).await.unwrap();

        let found = repo.find_by_id("ws1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Workspace");
    }

    #[tokio::test]
    async fn test_find_by_folder_path() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselWorkspaceRepository::new(pool);

        let workspace = create_test_workspace("ws2", "Path Test");
        repo.save(&workspace).await.unwrap();

        let found = repo.find_by_folder_path("/tmp/ws2").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "ws2");
    }

    #[tokio::test]
    async fn test_find_all_workspaces() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselWorkspaceRepository::new(pool);

        let ws1 = create_test_workspace("ws1", "Workspace 1");
        let ws2 = create_test_workspace("ws2", "Workspace 2");

        repo.save(&ws1).await.unwrap();
        repo.save(&ws2).await.unwrap();

        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_set_active_workspace() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselWorkspaceRepository::new(pool);

        let ws1 = create_test_workspace("ws1", "Workspace 1");
        let ws2 = create_test_workspace("ws2", "Workspace 2");

        repo.save(&ws1).await.unwrap();
        repo.save(&ws2).await.unwrap();

        repo.set_active("ws1").await.unwrap();

        let active = repo.find_active().await.unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, "ws1");

        // Set another workspace as active
        repo.set_active("ws2").await.unwrap();

        let active = repo.find_active().await.unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, "ws2");
    }

    #[tokio::test]
    async fn test_delete_workspace() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselWorkspaceRepository::new(pool);

        let workspace = create_test_workspace("ws_del", "To Delete");
        repo.save(&workspace).await.unwrap();

        repo.delete("ws_del").await.unwrap();

        let found = repo.find_by_id("ws_del").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_workspace_exists() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselWorkspaceRepository::new(pool);

        let workspace = create_test_workspace("ws_exists", "Exists Test");
        repo.save(&workspace).await.unwrap();

        assert!(repo.exists("ws_exists").await.unwrap());
        assert!(!repo.exists("ws_nonexistent").await.unwrap());
    }
}

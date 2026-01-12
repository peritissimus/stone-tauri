//! Notebook Repository Implementation
//!
//! Diesel-based implementation of the Notebook repository port.

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use diesel::prelude::*;

use crate::domain::{
    entities::Notebook,
    errors::{DomainError, DomainResult},
    ports::outbound::{
        NotebookFindOptions, NotebookPositionUpdate, NotebookRepository, NotebookWithCount,
    },
};
use crate::shared::database::schema::{notebooks, notes};

use super::{
    db_pool::{get_connection, DbPool},
    mappers::{InsertableNotebook, NotebookRow},
    utils::map_diesel_error,
};

/// Diesel implementation of NotebookRepository
pub struct DieselNotebookRepository {
    pool: Arc<DbPool>,
}

impl DieselNotebookRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Recursively get all ancestor IDs
    fn get_ancestors_recursive(
        conn: &mut SqliteConnection,
        notebook_id: &str,
        ancestors: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> DomainResult<()> {
        // Prevent infinite loops
        if visited.contains(notebook_id) {
            return Ok(());
        }
        visited.insert(notebook_id.to_string());

        // Find parent
        let notebook: Option<NotebookRow> = notebooks::table
            .filter(notebooks::id.eq(notebook_id))
            .first(conn)
            .optional()
            .map_err(map_diesel_error)?;

        if let Some(nb) = notebook {
            if let Some(parent_id) = nb.parent_id {
                ancestors.push(parent_id.clone());
                Self::get_ancestors_recursive(conn, &parent_id, ancestors, visited)?;
            }
        }

        Ok(())
    }

    /// Recursively get all descendant IDs
    fn get_descendants_recursive(
        conn: &mut SqliteConnection,
        notebook_id: &str,
        descendants: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> DomainResult<()> {
        // Prevent infinite loops
        if visited.contains(notebook_id) {
            return Ok(());
        }
        visited.insert(notebook_id.to_string());

        // Find children
        let children: Vec<NotebookRow> = notebooks::table
            .filter(notebooks::parent_id.eq(notebook_id))
            .load(conn)
            .map_err(map_diesel_error)?;

        for child in children {
            descendants.push(child.id.clone());
            Self::get_descendants_recursive(conn, &child.id, descendants, visited)?;
        }

        Ok(())
    }
}

#[async_trait]
impl NotebookRepository for DieselNotebookRepository {
    /// Find a notebook by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Notebook>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            notebooks::table
                .filter(notebooks::id.eq(id))
                .first::<NotebookRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find all notebooks matching the given options
    async fn find_all(&self, options: Option<NotebookFindOptions>) -> DomainResult<Vec<Notebook>> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notebooks::table.into_boxed();

            if let Some(opts) = options {
                if let Some(workspace_id) = opts.workspace_id {
                    query = query.filter(notebooks::workspace_id.eq(workspace_id));
                }

                if let Some(parent_filter) = opts.parent_id {
                    match parent_filter {
                        Some(parent_id) => {
                            query = query.filter(notebooks::parent_id.eq(parent_id));
                        }
                        None => {
                            query = query.filter(notebooks::parent_id.is_null());
                        }
                    }
                }
            }

            query = query.order((notebooks::position.asc(), notebooks::name.asc()));

            let rows = query.load::<NotebookRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find all notebooks with note counts
    async fn find_all_with_counts(
        &self,
        workspace_id: Option<&str>,
    ) -> DomainResult<Vec<NotebookWithCount>> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // First, get all notebooks
            let mut query = notebooks::table.into_boxed();

            if let Some(ws_id) = &workspace_id {
                query = query.filter(notebooks::workspace_id.eq(ws_id));
            }

            query = query.order((notebooks::position.asc(), notebooks::name.asc()));

            let notebook_rows: Vec<NotebookRow> = query
                .load(&mut conn)
                .map_err(map_diesel_error)?;

            // Then, count notes for each notebook
            let mut notebooks_with_counts = Vec::new();

            for notebook_row in notebook_rows {
                let notebook_id = notebook_row.id.clone();

                // Count notes for this notebook
                let note_count: i64 = notes::table
                    .filter(notes::notebook_id.eq(&notebook_id))
                    .filter(notes::is_deleted.eq(0))
                    .count()
                    .get_result(&mut conn)
                    .map_err(map_diesel_error)?;

                let notebook = notebook_row.to_domain();
                notebooks_with_counts.push(NotebookWithCount {
                    id: notebook.id,
                    name: notebook.name,
                    workspace_id: notebook.workspace_id,
                    parent_id: notebook.parent_id,
                    folder_path: notebook.folder_path,
                    position: notebook.position,
                    icon: Some(notebook.icon),
                    color: Some(notebook.color),
                    description: None,
                    created_at: notebook.created_at,
                    updated_at: notebook.updated_at,
                    note_count: note_count as i32,
                });
            }

            Ok(notebooks_with_counts)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find notebooks by workspace ID
    async fn find_by_workspace_id(&self, workspace_id: &str) -> DomainResult<Vec<Notebook>> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let rows = notebooks::table
                .filter(notebooks::workspace_id.eq(workspace_id))
                .order((notebooks::position.asc(), notebooks::name.asc()))
                .load::<NotebookRow>(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find notebooks by parent ID
    async fn find_by_parent_id(
        &self,
        parent_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> DomainResult<Vec<Notebook>> {
        let pool = self.pool.clone();
        let parent_id = parent_id.map(|s| s.to_string());
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notebooks::table.into_boxed();

            match parent_id {
                Some(pid) => {
                    query = query.filter(notebooks::parent_id.eq(pid));
                }
                None => {
                    query = query.filter(notebooks::parent_id.is_null());
                }
            }

            if let Some(ws_id) = workspace_id {
                query = query.filter(notebooks::workspace_id.eq(ws_id));
            }

            query = query.order((notebooks::position.asc(), notebooks::name.asc()));

            let rows = query.load::<NotebookRow>(&mut conn).map_err(map_diesel_error)?;

            Ok(rows.into_iter().map(|row| row.to_domain()).collect())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Find notebook by folder path
    async fn find_by_folder_path(
        &self,
        folder_path: &str,
        workspace_id: Option<&str>,
    ) -> DomainResult<Option<Notebook>> {
        let pool = self.pool.clone();
        let folder_path = folder_path.to_string();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notebooks::table.into_boxed();
            query = query.filter(notebooks::folder_path.eq(folder_path));

            if let Some(ws_id) = workspace_id {
                query = query.filter(notebooks::workspace_id.eq(ws_id));
            }

            query
                .first::<NotebookRow>(&mut conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(|row| Ok(row.to_domain()))
                .transpose()
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Save a notebook (create or update)
    async fn save(&self, notebook: &Notebook) -> DomainResult<()> {
        let pool = self.pool.clone();
        let notebook = notebook.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let insertable = InsertableNotebook::from_domain(&notebook);

            diesel::insert_into(notebooks::table)
                .values(&insertable)
                .on_conflict(notebooks::id)
                .do_update()
                .set(&insertable)
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Delete a notebook
    async fn delete(&self, id: &str) -> DomainResult<()> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let deleted = diesel::delete(notebooks::table.filter(notebooks::id.eq(&id)))
                .execute(&mut conn)
                .map_err(map_diesel_error)?;

            if deleted == 0 {
                return Err(DomainError::NotebookNotFound(id));
            }

            Ok(())
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get all ancestor IDs of a notebook (for preventing circular references)
    async fn get_ancestor_ids(&self, id: &str) -> DomainResult<Vec<String>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;
            let mut ancestors = Vec::new();
            let mut visited = HashSet::new();

            Self::get_ancestors_recursive(&mut conn, &id, &mut ancestors, &mut visited)?;

            Ok(ancestors)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Get all descendant IDs of a notebook
    async fn get_descendant_ids(&self, id: &str) -> DomainResult<Vec<String>> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;
            let mut descendants = Vec::new();
            let mut visited = HashSet::new();

            Self::get_descendants_recursive(&mut conn, &id, &mut descendants, &mut visited)?;

            Ok(descendants)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Check if a notebook exists
    async fn exists(&self, id: &str) -> DomainResult<bool> {
        let pool = self.pool.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let count: i64 = notebooks::table
                .filter(notebooks::id.eq(id))
                .count()
                .get_result(&mut conn)
                .map_err(map_diesel_error)?;

            Ok(count > 0)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Count notebooks in a workspace
    async fn count(&self, workspace_id: Option<&str>) -> DomainResult<i32> {
        let pool = self.pool.clone();
        let workspace_id = workspace_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            let mut query = notebooks::table.into_boxed();

            if let Some(ws_id) = workspace_id {
                query = query.filter(notebooks::workspace_id.eq(ws_id));
            }

            let count: i64 = query.count().get_result(&mut conn).map_err(map_diesel_error)?;

            Ok(count as i32)
        })
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Task join error: {}", e)))?
    }

    /// Update positions for reordering
    async fn update_positions(&self, updates: Vec<NotebookPositionUpdate>) -> DomainResult<()> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = get_connection(&pool)?;

            // Use transaction for atomicity
            conn.transaction(|conn| {
                for update in updates {
                    diesel::update(notebooks::table.filter(notebooks::id.eq(&update.id)))
                        .set(notebooks::position.eq(update.position))
                        .execute(conn)?;
                }
                Ok(())
            })
            .map_err(map_diesel_error)?;

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
    use chrono::Utc;

    fn create_test_notebook(id: &str, name: &str, parent_id: Option<String>) -> Notebook {
        Notebook {
            id: id.to_string(),
            name: name.to_string(),
            parent_id,
            workspace_id: Some("ws1".to_string()),
            folder_path: Some(format!("/notebooks/{}", id)),
            icon: "📁".to_string(),
            color: "#3b82f6".to_string(),
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_save_and_find_notebook() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNotebookRepository::new(pool);

        let notebook = create_test_notebook("nb1", "Test Notebook", None);
        repo.save(&notebook).await.unwrap();

        let found = repo.find_by_id("nb1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Notebook");
    }

    #[tokio::test]
    async fn test_hierarchical_notebooks() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNotebookRepository::new(pool);

        let parent = create_test_notebook("nb_parent", "Parent", None);
        let child = create_test_notebook("nb_child", "Child", Some("nb_parent".to_string()));

        repo.save(&parent).await.unwrap();
        repo.save(&child).await.unwrap();

        let children = repo.find_by_parent_id(Some("nb_parent"), None).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "nb_child");
    }

    #[tokio::test]
    async fn test_get_descendant_ids() {
        let pool = Arc::new(create_pool(":memory:").unwrap());
        let repo = DieselNotebookRepository::new(pool);

        let nb1 = create_test_notebook("nb1", "Root", None);
        let nb2 = create_test_notebook("nb2", "Child", Some("nb1".to_string()));
        let nb3 = create_test_notebook("nb3", "Grandchild", Some("nb2".to_string()));

        repo.save(&nb1).await.unwrap();
        repo.save(&nb2).await.unwrap();
        repo.save(&nb3).await.unwrap();

        let descendants = repo.get_descendant_ids("nb1").await.unwrap();
        assert_eq!(descendants.len(), 2);
        assert!(descendants.contains(&"nb2".to_string()));
        assert!(descendants.contains(&"nb3".to_string()));
    }
}

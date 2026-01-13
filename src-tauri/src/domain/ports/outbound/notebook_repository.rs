use crate::domain::{entities::Notebook, errors::DomainResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotebookFindOptions {
    pub workspace_id: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub include_note_count: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotebookWithCount {
    pub id: String,
    pub name: String,
    pub workspace_id: Option<String>,
    pub parent_id: Option<String>,
    pub folder_path: Option<String>,
    pub position: i32,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub note_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotebookPositionUpdate {
    pub id: String,
    pub position: i32,
}

/// Notebook Repository Port (Outbound)
///
/// Defines the contract for notebook persistence operations.
#[async_trait]
pub trait NotebookRepository: Send + Sync {
    /// Find a notebook by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Notebook>>;

    /// Find all notebooks matching the given options
    async fn find_all(&self, options: Option<NotebookFindOptions>) -> DomainResult<Vec<Notebook>>;

    /// Find all notebooks with note counts
    async fn find_all_with_counts(
        &self,
        workspace_id: Option<&str>,
    ) -> DomainResult<Vec<NotebookWithCount>>;

    /// Find notebooks by workspace ID
    async fn find_by_workspace_id(&self, workspace_id: &str) -> DomainResult<Vec<Notebook>>;

    /// Find notebooks by parent ID
    async fn find_by_parent_id(
        &self,
        parent_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> DomainResult<Vec<Notebook>>;

    /// Find notebook by folder path
    async fn find_by_folder_path(
        &self,
        folder_path: &str,
        workspace_id: Option<&str>,
    ) -> DomainResult<Option<Notebook>>;

    /// Save a notebook (create or update)
    async fn save(&self, notebook: &Notebook) -> DomainResult<()>;

    /// Delete a notebook
    async fn delete(&self, id: &str) -> DomainResult<()>;

    /// Get all ancestor IDs of a notebook (for preventing circular references)
    async fn get_ancestor_ids(&self, id: &str) -> DomainResult<Vec<String>>;

    /// Get all descendant IDs of a notebook
    async fn get_descendant_ids(&self, id: &str) -> DomainResult<Vec<String>>;

    /// Check if a notebook exists
    async fn exists(&self, id: &str) -> DomainResult<bool>;

    /// Count notebooks in a workspace
    async fn count(&self, workspace_id: Option<&str>) -> DomainResult<i32>;

    /// Update positions for reordering
    async fn update_positions(&self, updates: Vec<NotebookPositionUpdate>) -> DomainResult<()>;
}

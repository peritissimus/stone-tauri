use crate::domain::{entities::Notebook, errors::DomainResult, ports::outbound::NotebookWithCount};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNotebookRequest {
    pub name: String,
    pub parent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub folder_path: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotebookRequest {
    pub id: String,
    pub name: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNotebooksRequest {
    pub workspace_id: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub include_note_count: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNotebookRequest {
    pub id: String,
    pub delete_notes: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveNotebookRequest {
    pub id: String,
    pub target_parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotebookList {
    WithCount(Vec<NotebookWithCount>),
    WithoutCount(Vec<Notebook>),
}

/// Notebook Use Cases Port (Inbound)
///
/// Defines the contract for notebook-related use cases.
#[async_trait]
pub trait NotebookUseCases: Send + Sync {
    /// Create a new notebook
    async fn create_notebook(&self, request: CreateNotebookRequest) -> DomainResult<Notebook>;

    /// Update an existing notebook
    async fn update_notebook(&self, request: UpdateNotebookRequest) -> DomainResult<Notebook>;

    /// Get a notebook by ID
    async fn get_notebook(&self, id: &str) -> DomainResult<Notebook>;

    /// List notebooks with optional filtering
    async fn list_notebooks(&self, request: ListNotebooksRequest) -> DomainResult<NotebookList>;

    /// Delete a notebook
    async fn delete_notebook(&self, request: DeleteNotebookRequest) -> DomainResult<()>;

    /// Move a notebook to a different parent
    async fn move_notebook(&self, request: MoveNotebookRequest) -> DomainResult<()>;
}

use crate::domain::{entities::Workspace, errors::DomainResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub folder_path: String,
    pub set_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceRequest {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectFolderRequest {
    pub title: Option<String>,
    pub default_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectFolderResponse {
    pub canceled: bool,
    pub folder_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatePathResponse {
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanWorkspaceFileEntry {
    pub relative_path: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSystemEntryType {
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanWorkspaceFolderStructure {
    pub name: String,
    pub path: String,
    pub relative_path: String,
    #[serde(rename = "type")]
    pub entry_type: FileSystemEntryType,
    pub children: Option<Vec<ScanWorkspaceFolderStructure>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanWorkspaceResponse {
    pub files: Vec<ScanWorkspaceFileEntry>,
    pub structure: Vec<ScanWorkspaceFolderStructure>,
    pub total: i32,
    pub counts: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncWorkspaceStats {
    pub created: i32,
    pub updated: i32,
    pub deleted: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncWorkspaceResponse {
    pub notes: SyncWorkspaceStats,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderResponse {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFolderRequest {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFolderResponse {
    pub old_path: String,
    pub new_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveFolderRequest {
    pub source_path: String,
    pub destination_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveFolderResponse {
    pub old_path: String,
    pub new_path: String,
}

/// Workspace Use Cases Port (Inbound)
///
/// Defines the contract for workspace-related use cases.
#[async_trait]
pub trait WorkspaceUseCases: Send + Sync {
    /// Create a new workspace
    async fn create_workspace(&self, request: CreateWorkspaceRequest) -> DomainResult<Workspace>;

    /// Get a workspace by ID
    async fn get_workspace(&self, id: &str) -> DomainResult<Workspace>;

    /// List all workspaces
    async fn list_workspaces(&self) -> DomainResult<Vec<Workspace>>;

    /// Set active workspace
    async fn set_active_workspace(&self, id: &str) -> DomainResult<Workspace>;

    /// Get active workspace
    async fn get_active_workspace(&self) -> DomainResult<Option<Workspace>>;

    /// Delete a workspace
    async fn delete_workspace(&self, id: &str) -> DomainResult<()>;

    /// Update a workspace
    async fn update_workspace(&self, request: UpdateWorkspaceRequest) -> DomainResult<Workspace>;

    /// Select folder dialog
    async fn select_folder(
        &self,
        request: Option<SelectFolderRequest>,
    ) -> DomainResult<SelectFolderResponse>;

    /// Validate a path
    async fn validate_path(&self, path: &str) -> DomainResult<ValidatePathResponse>;

    /// Scan workspace for files
    async fn scan_workspace(&self, workspace_id: &str) -> DomainResult<ScanWorkspaceResponse>;

    /// Sync workspace with filesystem
    async fn sync_workspace(
        &self,
        workspace_id: Option<&str>,
    ) -> DomainResult<SyncWorkspaceResponse>;

    /// Create a folder within a workspace
    async fn create_folder(&self, request: CreateFolderRequest) -> DomainResult<CreateFolderResponse>;

    /// Rename a folder
    async fn rename_folder(&self, request: RenameFolderRequest) -> DomainResult<RenameFolderResponse>;

    /// Delete a folder
    async fn delete_folder(&self, path: &str) -> DomainResult<()>;

    /// Move a folder to a different location
    async fn move_folder(&self, request: MoveFolderRequest) -> DomainResult<MoveFolderResponse>;
}

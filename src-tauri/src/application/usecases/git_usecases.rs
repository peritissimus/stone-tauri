/// Git Use Cases Implementation
///
/// Application layer implementations for git operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{GitCommitInfo, GitStatusResponse, GitSyncResponse, GitUseCases},
        outbound::{git_service::GitFileStatus, GitService, WorkspaceRepository},
    },
};

/// Implementation of all Git use cases
pub struct GitUseCasesImpl {
    workspace_repository: Arc<dyn WorkspaceRepository>,
    git_service: Arc<dyn GitService>,
}

impl GitUseCasesImpl {
    pub fn new(
        workspace_repository: Arc<dyn WorkspaceRepository>,
        git_service: Arc<dyn GitService>,
    ) -> Self {
        Self {
            workspace_repository,
            git_service,
        }
    }
}

#[async_trait]
impl GitUseCases for GitUseCasesImpl {
    /// Get git status for a workspace
    async fn get_status(&self, workspace_id: &str) -> DomainResult<GitStatusResponse> {
        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Get git status - ✅ ASYNC
        let status = self
            .git_service
            .get_status(&workspace.folder_path)
            .await?;

        // Map GitStatus to GitStatusResponse
        let staged_count = status
            .changes
            .iter()
            .filter(|c| c.staged)
            .count() as i32;

        let unstaged_count = status
            .changes
            .iter()
            .filter(|c| !c.staged && c.status != GitFileStatus::Untracked)
            .count() as i32;

        let untracked_count = status
            .changes
            .iter()
            .filter(|c| c.status == GitFileStatus::Untracked)
            .count() as i32;

        Ok(GitStatusResponse {
            is_repo: status.is_repo,
            has_changes: status.has_uncommitted_changes,
            branch: status.branch,
            has_remote: status.remote_url.is_some(),
            remote_url: status.remote_url,
            ahead: status.ahead,
            behind: status.behind,
            staged: staged_count,
            unstaged: unstaged_count,
            untracked: untracked_count,
        })
    }

    /// Initialize git repository
    async fn init(&self, workspace_id: &str) -> DomainResult<bool> {
        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Initialize git - ✅ ASYNC
        let result = self.git_service.init(&workspace.folder_path).await?;

        Ok(result.success)
    }

    /// Commit changes
    async fn commit(
        &self,
        workspace_id: &str,
        message: Option<&str>,
    ) -> DomainResult<Option<GitCommitInfo>> {
        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Default commit message
        let default_message = format!("Commit: {}", chrono::Utc::now().to_rfc3339());
        let commit_message = message.unwrap_or(&default_message);

        // Stage all files - ✅ ASYNC
        self.git_service
            .stage(&workspace.folder_path, None)
            .await?;

        // Commit - ✅ ASYNC
        let result = self
            .git_service
            .commit(&workspace.folder_path, commit_message)
            .await?;

        // If commit failed, return None
        if !result.success {
            return Ok(None);
        }

        // Get latest commit info - ✅ ASYNC
        let commits = self
            .git_service
            .get_commits(&workspace.folder_path, Some(1))
            .await?;

        let latest = commits.first();

        Ok(latest.map(|c| GitCommitInfo {
            hash: c.hash.clone(),
            short_hash: c.short_hash.clone(),
            message: c.message.clone(),
            author: c.author.clone(),
            date: c.date,
        }))
    }

    /// Pull from remote
    async fn pull(&self, workspace_id: &str) -> DomainResult<bool> {
        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Pull - ✅ ASYNC
        let result = self.git_service.pull(&workspace.folder_path).await?;

        Ok(result.success)
    }

    /// Push to remote
    async fn push(&self, workspace_id: &str) -> DomainResult<bool> {
        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Push - ✅ ASYNC
        let result = self.git_service.push(&workspace.folder_path).await?;

        Ok(result.success)
    }

    /// Sync (pull + commit + push)
    async fn sync(
        &self,
        workspace_id: &str,
        message: Option<&str>,
    ) -> DomainResult<GitSyncResponse> {
        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Default sync message
        let default_message = format!("Sync: {}", chrono::Utc::now().to_rfc3339());
        let sync_message = message.unwrap_or(&default_message);

        // Sync - ✅ ASYNC
        let result = self
            .git_service
            .sync(&workspace.folder_path, Some(sync_message))
            .await?;

        Ok(GitSyncResponse {
            success: result.success,
            pulled: 0,
            pushed: 0,
            conflicts: vec![],
            error: result.error,
        })
    }

    /// Set remote URL
    async fn set_remote(&self, workspace_id: &str, url: &str) -> DomainResult<bool> {
        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Set remote - ✅ ASYNC
        let result = self
            .git_service
            .set_remote(&workspace.folder_path, url, Some("origin"))
            .await?;

        Ok(result.success)
    }

    /// Get recent commits
    async fn get_commits(
        &self,
        workspace_id: &str,
        limit: Option<i32>,
    ) -> DomainResult<Vec<GitCommitInfo>> {
        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Get commits - ✅ ASYNC
        let commits = self
            .git_service
            .get_commits(&workspace.folder_path, limit.or(Some(50)))
            .await?;

        // Map GitCommit to GitCommitInfo
        let commit_infos = commits
            .into_iter()
            .map(|c| GitCommitInfo {
                hash: c.hash,
                short_hash: c.short_hash,
                message: c.message,
                author: c.author,
                date: c.date,
            })
            .collect();

        Ok(commit_infos)
    }
}

//! Git Service Implementation
//!
//! Implementation of Git operations using the git2 crate (libgit2).

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use git2::{
    BranchType, Commit, Cred, FetchOptions, IndexAddOption, PushOptions,
    RemoteCallbacks, Repository, Signature, StatusOptions,
};
use std::path::Path;
use tracing::{error, info, warn};

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::outbound::{
        GitCommit, GitFileChange, GitFileStatus, GitOperationResult, GitService, GitStatus,
    },
};

/// Git2-based implementation of GitService
pub struct Git2Service;

impl Git2Service {
    pub fn new() -> Self {
        Self
    }

    /// Convert git2 Error to DomainError
    fn map_git_error(err: git2::Error) -> DomainError {
        DomainError::ExternalServiceError(format!("Git error: {}", err))
    }

    /// Get repository from path
    fn get_repo(path: &str) -> DomainResult<Repository> {
        Repository::open(path).map_err(Self::map_git_error)
    }

    /// Convert git2 Commit to domain GitCommit
    fn commit_to_domain(commit: &Commit) -> GitCommit {
        let time = commit.time();
        let timestamp = time.seconds();
        let date = Utc.timestamp_opt(timestamp, 0).unwrap();

        GitCommit {
            hash: commit.id().to_string(),
            short_hash: commit.id().to_string()[..7].to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            email: commit.author().email().unwrap_or("").to_string(),
            date,
        }
    }

    /// Create remote callbacks with authentication
    fn create_remote_callbacks<'a>() -> RemoteCallbacks<'a> {
        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(|_url, username_from_url, allowed_types| {
            let username = username_from_url.unwrap_or("git");
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

            // Try SSH key file FIRST (more reliable than agent)
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                let ssh_key_ed25519 = format!("{}/.ssh/id_ed25519", home);
                let ssh_key_rsa = format!("{}/.ssh/id_rsa", home);
                let pub_key_ed25519 = format!("{}/.ssh/id_ed25519.pub", home);
                let pub_key_rsa = format!("{}/.ssh/id_rsa.pub", home);

                // Try id_ed25519 first
                if std::path::Path::new(&ssh_key_ed25519).exists() {
                    let pub_key = if std::path::Path::new(&pub_key_ed25519).exists() {
                        Some(std::path::Path::new(&pub_key_ed25519))
                    } else {
                        None
                    };
                    if let Ok(cred) = Cred::ssh_key(username, pub_key, std::path::Path::new(&ssh_key_ed25519), None) {
                        return Ok(cred);
                    }
                }

                // Try id_rsa
                if std::path::Path::new(&ssh_key_rsa).exists() {
                    let pub_key = if std::path::Path::new(&pub_key_rsa).exists() {
                        Some(std::path::Path::new(&pub_key_rsa))
                    } else {
                        None
                    };
                    if let Ok(cred) = Cred::ssh_key(username, pub_key, std::path::Path::new(&ssh_key_rsa), None) {
                        return Ok(cred);
                    }
                }

                // Try SSH agent as fallback
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }
            }

            // Try default credentials (for HTTPS)
            if allowed_types.contains(git2::CredentialType::DEFAULT) {
                if let Ok(cred) = Cred::default() {
                    return Ok(cred);
                }
            }

            error!("[Git] Auth failed. Run 'ssh-add ~/.ssh/id_ed25519' to add your key.");
            Err(git2::Error::from_str("Authentication failed - run 'ssh-add' to add your SSH key"))
        });

        callbacks
    }
}

impl Default for Git2Service {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GitService for Git2Service {
    async fn is_repository(&self, path: &str) -> DomainResult<bool> {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || Ok(Repository::open(&path).is_ok()))
            .await
            .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn init(&self, path: &str) -> DomainResult<GitOperationResult> {
        let path = path.to_string();

        tokio::task::spawn_blocking(move || {
            match Repository::init(&path) {
                Ok(_) => Ok(GitOperationResult {
                    success: true,
                    message: Some("Repository initialized successfully".to_string()),
                    error: None,
                }),
                Err(e) => Ok(GitOperationResult {
                    success: false,
                    message: None,
                    error: Some(format!("Failed to initialize repository: {}", e)),
                }),
            }
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn get_status(&self, path: &str) -> DomainResult<GitStatus> {
        let path = path.to_string();

        tokio::task::spawn_blocking(move || {
            let repo = Self::get_repo(&path)?;
            let is_repo = true;

            let head = repo.head().ok();
            let branch = head.as_ref().and_then(|h| h.shorthand().map(|s| s.to_string()));

            let remote = repo.find_remote("origin").ok();
            let has_remote = remote.is_some();
            let remote_url = remote.and_then(|r| r.url().map(|s| s.to_string()));

            let (ahead, behind) = if let (Some(local_oid), Some(upstream_oid)) = (
                head.as_ref().and_then(|h| h.target()),
                repo.find_branch("origin/main", BranchType::Remote)
                    .or_else(|_| repo.find_branch("origin/master", BranchType::Remote))
                    .ok()
                    .and_then(|b| b.get().target()),
            ) {
                repo.graph_ahead_behind(local_oid, upstream_oid).unwrap_or((0, 0))
            } else {
                (0, 0)
            };

            let mut status_opts = StatusOptions::new();
            status_opts.include_untracked(true);
            status_opts.recurse_untracked_dirs(true);

            let statuses = repo.statuses(Some(&mut status_opts)).map_err(Self::map_git_error)?;
            let mut changes = Vec::new();
            let has_uncommitted_changes = !statuses.is_empty();

            for entry in statuses.iter() {
                let status = entry.status();
                let path = entry.path().unwrap_or("").to_string();

                if status.is_wt_new() || status.is_index_new() {
                    changes.push(GitFileChange { path, status: GitFileStatus::Added, staged: status.is_index_new() });
                } else if status.is_wt_modified() || status.is_index_modified() {
                    changes.push(GitFileChange { path, status: GitFileStatus::Modified, staged: status.is_index_modified() });
                } else if status.is_wt_deleted() || status.is_index_deleted() {
                    changes.push(GitFileChange { path, status: GitFileStatus::Deleted, staged: status.is_index_deleted() });
                } else if status.is_wt_renamed() || status.is_index_renamed() {
                    changes.push(GitFileChange { path, status: GitFileStatus::Renamed, staged: status.is_index_renamed() });
                }
            }

            Ok(GitStatus { is_repo, branch, ahead: ahead as i32, behind: behind as i32, has_remote, remote_url, changes, has_uncommitted_changes })
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn stage(&self, path: &str, files: Option<Vec<String>>) -> DomainResult<GitOperationResult> {
        let path = path.to_string();

        tokio::task::spawn_blocking(move || {
            let repo = Self::get_repo(&path)?;
            let mut index = repo.index().map_err(Self::map_git_error)?;

            match files {
                Some(file_list) => {
                    for file in file_list {
                        index.add_path(Path::new(&file)).map_err(Self::map_git_error)?;
                    }
                }
                None => {
                    index.add_all(["."].iter(), IndexAddOption::DEFAULT, None).map_err(Self::map_git_error)?;
                }
            }

            index.write().map_err(Self::map_git_error)?;
            Ok(GitOperationResult { success: true, message: Some("Files staged".to_string()), error: None })
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn commit(&self, path: &str, message: &str) -> DomainResult<GitOperationResult> {
        let path = path.to_string();
        let message = message.to_string();

        tokio::task::spawn_blocking(move || {
            let repo = Self::get_repo(&path)?;
            let mut index = repo.index().map_err(Self::map_git_error)?;
            let tree_id = index.write_tree().map_err(Self::map_git_error)?;
            let tree = repo.find_tree(tree_id).map_err(Self::map_git_error)?;

            let signature = repo.signature()
                .or_else(|_| Signature::now("Stone User", "stone@localhost"))
                .map_err(Self::map_git_error)?;

            let parent_commit = repo.head().ok().and_then(|h| h.target()).and_then(|oid| repo.find_commit(oid).ok());
            let parents = if let Some(ref parent) = parent_commit { vec![parent] } else { vec![] };

            let commit_oid = repo.commit(Some("HEAD"), &signature, &signature, &message, &tree, &parents)
                .map_err(Self::map_git_error)?;

            Ok(GitOperationResult { success: true, message: Some(format!("Committed: {}", commit_oid)), error: None })
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn pull(&self, path: &str) -> DomainResult<GitOperationResult> {
        let path = path.to_string();

        tokio::task::spawn_blocking(move || {
            let repo = Self::get_repo(&path)?;

            let head = repo.head().map_err(Self::map_git_error)?;
            let branch_name = head.shorthand()
                .ok_or_else(|| DomainError::ExternalServiceError("Could not get branch name".to_string()))?
                .to_string();

            let mut remote = repo.find_remote("origin").map_err(Self::map_git_error)?;
            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(Self::create_remote_callbacks());

            remote.fetch(&[&branch_name], Some(&mut fetch_options), None).map_err(Self::map_git_error)?;

            let fetch_head = match repo.find_reference("FETCH_HEAD") {
                Ok(fh) => fh,
                Err(_) => return Ok(GitOperationResult { success: true, message: Some("Already up to date".to_string()), error: None }),
            };

            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).map_err(Self::map_git_error)?;
            let (merge_analysis, _) = repo.merge_analysis(&[&fetch_commit]).map_err(Self::map_git_error)?;

            if merge_analysis.is_up_to_date() {
                return Ok(GitOperationResult { success: true, message: Some("Already up to date".to_string()), error: None });
            }

            if merge_analysis.is_fast_forward() {
                let refname = format!("refs/heads/{}", branch_name);
                let mut reference = repo.find_reference(&refname).map_err(Self::map_git_error)?;
                reference.set_target(fetch_commit.id(), "Fast-forward pull").map_err(Self::map_git_error)?;
                repo.set_head(&refname).map_err(Self::map_git_error)?;
                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())).map_err(Self::map_git_error)?;
                return Ok(GitOperationResult { success: true, message: Some("Fast-forward merge completed".to_string()), error: None });
            }

            if merge_analysis.is_normal() {
                warn!("[Git] Merge conflict detected");
                return Ok(GitOperationResult { success: false, message: None, error: Some("Merge conflict - please resolve manually".to_string()) });
            }

            Ok(GitOperationResult { success: true, message: Some("Pulled".to_string()), error: None })
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn push(&self, path: &str) -> DomainResult<GitOperationResult> {
        let path = path.to_string();

        tokio::task::spawn_blocking(move || {
            let repo = Self::get_repo(&path)?;
            let mut remote = repo.find_remote("origin").map_err(Self::map_git_error)?;

            let mut push_options = PushOptions::new();
            push_options.remote_callbacks(Self::create_remote_callbacks());

            let branch = repo.head().ok()
                .and_then(|h| h.shorthand().map(|s| s.to_string()))
                .unwrap_or_else(|| "main".to_string());

            let refspec = format!("refs/heads/{}", branch);
            remote.push(&[&refspec], Some(&mut push_options)).map_err(Self::map_git_error)?;

            Ok(GitOperationResult { success: true, message: Some("Pushed".to_string()), error: None })
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn set_remote(
        &self,
        path: &str,
        url: &str,
        name: Option<&str>,
    ) -> DomainResult<GitOperationResult> {
        let path = path.to_string();
        let url = url.to_string();
        let name = name.unwrap_or("origin").to_string();

        tokio::task::spawn_blocking(move || {
            let repo = Self::get_repo(&path)?;

            // Try to find existing remote
            if repo.find_remote(&name).is_ok() {
                repo.remote_set_url(&name, &url)
                    .map_err(Self::map_git_error)?;
            } else {
                repo.remote(&name, &url)
                    .map_err(Self::map_git_error)?;
            }

            Ok(GitOperationResult {
                success: true,
                message: Some(format!("Remote '{}' set to {}", name, url)),
                error: None,
            })
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn get_commits(&self, path: &str, limit: Option<i32>) -> DomainResult<Vec<GitCommit>> {
        let path = path.to_string();
        let limit = limit.unwrap_or(10) as usize;

        tokio::task::spawn_blocking(move || {
            let repo = Self::get_repo(&path)?;
            let mut revwalk = repo.revwalk().map_err(Self::map_git_error)?;

            revwalk.push_head().map_err(Self::map_git_error)?;

            let mut commits = Vec::new();

            for (idx, oid_result) in revwalk.enumerate() {
                if idx >= limit {
                    break;
                }

                let oid = oid_result.map_err(Self::map_git_error)?;
                let commit = repo.find_commit(oid).map_err(Self::map_git_error)?;
                commits.push(Self::commit_to_domain(&commit));
            }

            Ok(commits)
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn sync(&self, path: &str, message: Option<&str>) -> DomainResult<GitOperationResult> {
        info!("[Git] Sync started");
        let path_str = path.to_string();
        let commit_message = message.unwrap_or("Auto-sync").to_string();

        // Pull first
        let pull_result = self.pull(&path_str).await?;
        if !pull_result.success {
            return Ok(pull_result);
        }

        // Check and commit local changes
        let status = self.get_status(&path_str).await?;
        if status.has_uncommitted_changes {
            self.stage(&path_str, None).await?;
            self.commit(&path_str, &commit_message).await?;
        }

        // Push
        let push_result = self.push(&path_str).await?;
        info!("[Git] Sync completed");
        Ok(push_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_repository() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let service = Git2Service::new();
        let result = service.init(path).await.unwrap();

        assert!(result.success);
        assert!(service.is_repository(path).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_repository_false() {
        let service = Git2Service::new();
        let result = service.is_repository("/nonexistent/path").await.unwrap();
        assert!(!result);
    }
}

//! File Watcher Implementation
//!
//! Implementation using the notify crate for cross-platform file system watching.

use async_trait::async_trait;
use notify::{
    event::{EventKind, ModifyKind},
    Config, Event, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

use crate::domain::{
    entities::Workspace,
    errors::{DomainError, DomainResult},
    ports::outbound::{EventPublisher, FileWatcher},
};

/// File watcher implementation using notify crate
pub struct NotifyFileWatcher {
    /// Active watchers per workspace
    watchers: Arc<Mutex<HashMap<String, RecommendedWatcher>>>,
    /// Event publisher for file change events
    event_publisher: Arc<dyn EventPublisher>,
    /// Debounce duration for file events (milliseconds)
    debounce_ms: u64,
}

impl NotifyFileWatcher {
    pub fn new(event_publisher: Arc<dyn EventPublisher>) -> Self {
        Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            event_publisher,
            debounce_ms: 300, // 300ms debounce
        }
    }

    pub fn with_debounce(event_publisher: Arc<dyn EventPublisher>, debounce_ms: u64) -> Self {
        Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            event_publisher,
            debounce_ms,
        }
    }

    /// Check if a file should be ignored (temporary files, etc.)
    fn should_ignore_file(path: &Path) -> bool {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Ignore hidden files, temporary files, and system files
            if file_name.starts_with('.')
                || file_name.starts_with('~')
                || file_name.ends_with('~')
                || file_name.ends_with(".swp")
                || file_name.ends_with(".tmp")
                || file_name.ends_with(".DS_Store")
                || file_name == "Thumbs.db"
            {
                return true;
            }
        }

        // Ignore certain directories
        if let Some(parent) = path.parent() {
            if let Some(dir_name) = parent.file_name().and_then(|n| n.to_str()) {
                if dir_name == ".git"
                    || dir_name == "node_modules"
                    || dir_name == "target"
                    || dir_name == ".idea"
                {
                    return true;
                }
            }
        }

        false
    }

    /// Process file system events
    fn process_event(&self, event: Event) {
        use crate::domain::ports::outbound::{DomainEvent, FileSyncOperation};

        let kind = event.kind;
        for path in event.paths {
            // Skip ignored files
            if Self::should_ignore_file(&path) {
                continue;
            }

            let file_path = path.to_string_lossy().to_string();

            let operation = match kind {
                EventKind::Create(_) => FileSyncOperation::Created,
                EventKind::Modify(ModifyKind::Data(_)) => FileSyncOperation::Updated,
                EventKind::Remove(_) => FileSyncOperation::Deleted,
                _ => continue, // Skip other event types
            };

            // Publish file sync event
            self.event_publisher.publish(DomainEvent::FileSynced {
                timestamp: chrono::Utc::now(),
                file_path,
                operation,
            });
        }
    }
}

#[async_trait]
impl FileWatcher for NotifyFileWatcher {
    async fn start(&self) -> DomainResult<()> {
        // This method would typically load all workspaces from a repository
        // and start watching them. For now, it's a no-op as workspaces
        // are watched individually via watch_workspace
        Ok(())
    }

    async fn stop_all(&self) -> DomainResult<()> {
        let mut watchers = self.watchers.lock().await;
        watchers.clear(); // Dropping watchers stops them
        Ok(())
    }

    async fn watch_workspace(&self, workspace: &Workspace) -> DomainResult<()> {
        let workspace_id = workspace.id.clone();
        let folder_path = workspace.folder_path.clone();

        // Check if already watching
        {
            let watchers = self.watchers.lock().await;
            if watchers.contains_key(&workspace_id) {
                return Ok(()); // Already watching
            }
        }

        // Create event channel
        let (tx, mut rx): (Sender<Result<Event, notify::Error>>, Receiver<Result<Event, notify::Error>>) =
            channel(1000);

        // Create watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(Ok(event));
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(self.debounce_ms)),
        )
        .map_err(|e| DomainError::ExternalServiceError(format!("Failed to create watcher: {}", e)))?;

        // Start watching the workspace folder
        watcher
            .watch(Path::new(&folder_path), RecursiveMode::Recursive)
            .map_err(|e| {
                DomainError::ExternalServiceError(format!("Failed to watch folder: {}", e))
            })?;

        // Store the watcher
        {
            let mut watchers = self.watchers.lock().await;
            watchers.insert(workspace_id.clone(), watcher);
        }

        // Spawn task to process events
        let event_publisher = self.event_publisher.clone();
        let workspace_id_clone = workspace_id.clone();

        tokio::spawn(async move {
            while let Some(event_result) = rx.recv().await {
                if let Ok(event) = event_result {
                    // Create a temporary NotifyFileWatcher instance to access process_event
                    // This is a bit awkward but necessary due to the async nature
                    let temp_watcher = NotifyFileWatcher {
                        watchers: Arc::new(Mutex::new(HashMap::new())),
                        event_publisher: event_publisher.clone(),
                        debounce_ms: 300,
                    };
                    temp_watcher.process_event(event);
                }
            }

            tracing::debug!("File watcher stopped for workspace: {}", workspace_id_clone);
        });

        Ok(())
    }

    async fn unwatch_workspace(&self, workspace_id: &str) -> DomainResult<()> {
        let mut watchers = self.watchers.lock().await;

        if watchers.remove(workspace_id).is_some() {
            Ok(())
        } else {
            Err(DomainError::NotFound(format!(
                "No watcher found for workspace: {}",
                workspace_id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::services::event_publisher_impl::TokioEventPublisher;
    use chrono::Utc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;
    use tokio::fs;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_watch_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let event_publisher = Arc::new(TokioEventPublisher::new());
        let watcher = NotifyFileWatcher::new(event_publisher.clone());

        let workspace = Workspace {
            id: "test-workspace".to_string(),
            name: "Test".to_string(),
            folder_path: temp_dir.path().to_string_lossy().to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_opened_at: Some(Utc::now()),
            icon: None,
            git_enabled: false,
            file_watcher_enabled: true,
        };

        let result = watcher.watch_workspace(&workspace).await;
        assert!(result.is_ok());

        // Check that workspace is being watched
        let watchers = watcher.watchers.lock().await;
        assert!(watchers.contains_key("test-workspace"));
    }

    #[tokio::test]
    async fn test_unwatch_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let event_publisher = Arc::new(TokioEventPublisher::new());
        let watcher = NotifyFileWatcher::new(event_publisher.clone());

        let workspace = Workspace {
            id: "test-workspace".to_string(),
            name: "Test".to_string(),
            folder_path: temp_dir.path().to_string_lossy().to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_opened_at: Some(Utc::now()),
            icon: None,
            git_enabled: false,
            file_watcher_enabled: true,
        };

        watcher.watch_workspace(&workspace).await.unwrap();

        // Unwatch the workspace
        let result = watcher.unwatch_workspace("test-workspace").await;
        assert!(result.is_ok());

        // Check that workspace is no longer being watched
        let watchers = watcher.watchers.lock().await;
        assert!(!watchers.contains_key("test-workspace"));
    }

    #[tokio::test]
    async fn test_stop_all() {
        let temp_dir = TempDir::new().unwrap();
        let event_publisher = Arc::new(TokioEventPublisher::new());
        let watcher = NotifyFileWatcher::new(event_publisher.clone());

        let workspace = Workspace {
            id: "test-workspace".to_string(),
            name: "Test".to_string(),
            folder_path: temp_dir.path().to_string_lossy().to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_opened_at: Some(Utc::now()),
            icon: None,
            git_enabled: false,
            file_watcher_enabled: true,
        };

        watcher.watch_workspace(&workspace).await.unwrap();

        // Stop all watchers
        watcher.stop_all().await.unwrap();

        // Check that all watchers are stopped
        let watchers = watcher.watchers.lock().await;
        assert!(watchers.is_empty());
    }

    #[test]
    fn test_should_ignore_file() {
        assert!(NotifyFileWatcher::should_ignore_file(Path::new(".hidden")));
        assert!(NotifyFileWatcher::should_ignore_file(Path::new("~tempfile")));
        assert!(NotifyFileWatcher::should_ignore_file(Path::new("file.swp")));
        assert!(NotifyFileWatcher::should_ignore_file(Path::new(".DS_Store")));
        assert!(!NotifyFileWatcher::should_ignore_file(Path::new("note.md")));
    }
}

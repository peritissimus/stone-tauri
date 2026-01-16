/// Workspace Use Cases Implementation
///
/// Application layer implementations for workspace operations.
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    entities::{Note, Workspace},
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{
            CreateFolderRequest, CreateFolderResponse, CreateWorkspaceRequest,
            FileSystemEntryType, MoveFolderRequest, MoveFolderResponse, RenameFolderRequest,
            RenameFolderResponse, ScanWorkspaceFolderStructure, ScanWorkspaceFileEntry,
            ScanWorkspaceResponse, SelectFolderRequest, SelectFolderResponse,
            SyncWorkspaceResponse, SyncWorkspaceStats, UpdateWorkspaceRequest,
            ValidatePathResponse, WorkspaceUseCases,
        },
        outbound::{
            EventPublisher, FileStorage, FolderPickerOptions, MarkdownProcessor, NoteRepository,
            NoteFindOptions, SystemService, WorkspaceRepository,
        },
    },
};

/// Implementation of all Workspace use cases
pub struct WorkspaceUseCasesImpl {
    workspace_repository: Arc<dyn WorkspaceRepository>,
    note_repository: Arc<dyn NoteRepository>,
    file_storage: Arc<dyn FileStorage>,
    system_service: Arc<dyn SystemService>,
    markdown_processor: Arc<dyn MarkdownProcessor>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl WorkspaceUseCasesImpl {
    pub fn new(
        workspace_repository: Arc<dyn WorkspaceRepository>,
        note_repository: Arc<dyn NoteRepository>,
        file_storage: Arc<dyn FileStorage>,
        system_service: Arc<dyn SystemService>,
        markdown_processor: Arc<dyn MarkdownProcessor>,
        event_publisher: Option<Arc<dyn EventPublisher>>,
    ) -> Self {
        Self {
            workspace_repository,
            note_repository,
            file_storage,
            system_service,
            markdown_processor,
            event_publisher,
        }
    }

    /// Build folder structure recursively for scan operation
    fn build_folder_structure<'a>(
        &'a self,
        base_path: &'a str,
        relative_path: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = DomainResult<Vec<ScanWorkspaceFolderStructure>>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
        let current_path = if relative_path.is_empty() {
            base_path.to_string()
        } else {
            Path::new(base_path).join(relative_path).to_string_lossy().to_string()
        };

        let items = self.file_storage.list_files(&current_path).await?;
        let mut result = Vec::new();

        for item in items {
            // Skip hidden files/folders
            if item.name.starts_with('.') {
                continue;
            }

            let item_relative_path = if relative_path.is_empty() {
                item.name.clone()
            } else {
                format!("{}/{}", relative_path, item.name)
            };

            if item.is_directory {
                let children = self.build_folder_structure(base_path, &item_relative_path).await?;
                result.push(ScanWorkspaceFolderStructure {
                    name: item.name,
                    path: item.path,
                    relative_path: item_relative_path.replace('\\', "/"),
                    entry_type: FileSystemEntryType::Folder,
                    children: Some(children),
                });
            } else if item.name.ends_with(".md") {
                result.push(ScanWorkspaceFolderStructure {
                    name: item.name,
                    path: item.path,
                    relative_path: item_relative_path.replace('\\', "/"),
                    entry_type: FileSystemEntryType::File,
                    children: None,
                });
            }
        }

        // Sort the result:
        // - Folders first, then files
        // - For Journal folder files (YYYY-MM-DD.md), sort by date descending (newest first)
        // - For everything else, sort alphabetically
        result.sort_by(|a, b| {
            use std::cmp::Ordering;

            // Folders before files
            match (&a.entry_type, &b.entry_type) {
                (FileSystemEntryType::Folder, FileSystemEntryType::File) => return Ordering::Less,
                (FileSystemEntryType::File, FileSystemEntryType::Folder) => return Ordering::Greater,
                _ => {}
            }

            // Both are files in Journal folder - sort by date descending
            if relative_path == "Journal" && a.entry_type == FileSystemEntryType::File && b.entry_type == FileSystemEntryType::File {
                // Extract dates from filenames (format: YYYY-MM-DD.md)
                let a_date = a.name.strip_suffix(".md").unwrap_or(&a.name);
                let b_date = b.name.strip_suffix(".md").unwrap_or(&b.name);

                // Compare as strings in reverse (descending order)
                // Since format is YYYY-MM-DD, lexicographic comparison works
                return b_date.cmp(a_date);
            }

            // Default: alphabetical order
            a.name.cmp(&b.name)
        });

        Ok(result)
        })
    }
}

#[async_trait]
impl WorkspaceUseCases for WorkspaceUseCasesImpl {
    /// Create a new workspace
    async fn create_workspace(
        &self,
        request: CreateWorkspaceRequest,
    ) -> DomainResult<Workspace> {
        let mut workspace = Workspace::new(request.name, request.folder_path)?;

        if request.set_active.unwrap_or(false) {
            workspace.activate();
        }

        self.workspace_repository.save(&workspace).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "workspace:created",
                serde_json::json!({"id": workspace.id}),
            );
        }

        Ok(workspace)
    }

    /// Get a workspace by ID
    async fn get_workspace(&self, id: &str) -> DomainResult<Workspace> {
        self.workspace_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::WorkspaceNotFound(id.to_string()))
    }

    /// List all workspaces
    async fn list_workspaces(&self) -> DomainResult<Vec<Workspace>> {
        self.workspace_repository.find_all().await
    }

    /// Set active workspace
    async fn set_active_workspace(&self, id: &str) -> DomainResult<Workspace> {
        let workspace = self
            .workspace_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::WorkspaceNotFound(id.to_string()))?;

        // Deactivate all workspaces
        let all_workspaces = self.workspace_repository.find_all().await?;
        for mut ws in all_workspaces {
            if ws.is_active && ws.id != id {
                ws.deactivate();
                self.workspace_repository.save(&ws).await?;
            }
        }

        // Activate the requested workspace
        let mut active_workspace = workspace;
        active_workspace.activate();
        self.workspace_repository.save(&active_workspace).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "workspace:switched",
                serde_json::json!({"id": active_workspace.id}),
            );
        }

        Ok(active_workspace)
    }

    /// Get active workspace
    async fn get_active_workspace(&self) -> DomainResult<Option<Workspace>> {
        self.workspace_repository.find_active().await
    }

    /// Delete a workspace
    async fn delete_workspace(&self, id: &str) -> DomainResult<()> {
        if !self.workspace_repository.exists(id).await? {
            return Err(DomainError::WorkspaceNotFound(id.to_string()));
        }

        self.workspace_repository.delete(id).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit("workspace:deleted", serde_json::json!({"id": id}));
        }

        Ok(())
    }

    /// Update a workspace
    async fn update_workspace(
        &self,
        request: UpdateWorkspaceRequest,
    ) -> DomainResult<Workspace> {
        let mut workspace = self
            .workspace_repository
            .find_by_id(&request.id)
            .await?
            .ok_or_else(|| DomainError::WorkspaceNotFound(request.id.clone()))?;

        if let Some(name) = request.name {
            workspace.rename(name)?;
        }

        self.workspace_repository.save(&workspace).await?;

        // Publish event
        if let Some(ref publisher) = self.event_publisher {
            publisher.emit(
                "workspace:updated",
                serde_json::json!({"id": workspace.id}),
            );
        }

        Ok(workspace)
    }

    /// Select folder dialog
    async fn select_folder(
        &self,
        request: Option<SelectFolderRequest>,
    ) -> DomainResult<SelectFolderResponse> {
        let options = request.as_ref().map(|r| FolderPickerOptions {
            title: r.title.clone(),
            default_path: r.default_path.clone(),
            button_label: Some("Select Folder".to_string()),
        });

        let folder_path = self.system_service.select_folder(options).await?;

        if let Some(path) = folder_path {
            Ok(SelectFolderResponse {
                canceled: false,
                folder_path: Some(path),
            })
        } else {
            Ok(SelectFolderResponse {
                canceled: true,
                folder_path: None,
            })
        }
    }

    /// Validate a path
    async fn validate_path(&self, path: &str) -> DomainResult<ValidatePathResponse> {
        let is_valid = self.system_service.validate_path(path).await?;

        if is_valid {
            Ok(ValidatePathResponse {
                valid: true,
                error: None,
            })
        } else {
            Ok(ValidatePathResponse {
                valid: false,
                error: Some("Path does not exist or is not accessible".to_string()),
            })
        }
    }

    /// Scan workspace for files
    async fn scan_workspace(&self, workspace_id: &str) -> DomainResult<ScanWorkspaceResponse> {
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| DomainError::WorkspaceNotFound(workspace_id.to_string()))?;

        // Get all markdown files
        let markdown_paths = self
            .file_storage
            .glob("**/*.md", &workspace.folder_path)
            .await?;

        // Build files array with relativePath and absolute path
        let files: Vec<ScanWorkspaceFileEntry> = markdown_paths
            .iter()
            .map(|relative_path| {
                let path = Path::new(&workspace.folder_path)
                    .join(relative_path)
                    .to_string_lossy()
                    .to_string();
                ScanWorkspaceFileEntry {
                    relative_path: relative_path.replace('\\', "/"),
                    path,
                }
            })
            .collect();

        // Build folder structure tree
        let structure = self.build_folder_structure(&workspace.folder_path, "").await?;

        // Build counts per folder
        let mut counts: HashMap<String, i32> = HashMap::new();
        let total_files = files.len() as i32;
        counts.insert("__root__".to_string(), total_files);

        for file in &files {
            let parts: Vec<&str> = file.relative_path.split('/').collect();
            if parts.len() > 1 {
                let mut current_path = String::new();
                for part in parts.iter().take(parts.len() - 1) {
                    current_path = if current_path.is_empty() {
                        part.to_string()
                    } else {
                        format!("{}/{}", current_path, part)
                    };
                    *counts.entry(current_path.clone()).or_insert(0) += 1;
                }
            }
        }
        
        // Ensure counts are finite - though with i32 they shouldn't be NaN
        // This is a safety measure against weird serialization behavior
        let sanitized_counts: HashMap<String, i32> = counts
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect();

        Ok(ScanWorkspaceResponse {
            files,
            structure,
            total: total_files,
            counts: sanitized_counts,
        })
    }

    /// Sync workspace with filesystem
    async fn sync_workspace(
        &self,
        workspace_id: Option<&str>,
    ) -> DomainResult<SyncWorkspaceResponse> {
        let start_time = std::time::Instant::now();

        let workspace = if let Some(id) = workspace_id {
            self.workspace_repository
                .find_by_id(id)
                .await?
                .ok_or_else(|| DomainError::WorkspaceNotFound(id.to_string()))?
        } else {
            self.workspace_repository
                .find_active()
                .await?
                .ok_or_else(|| DomainError::ValidationError("No active workspace".to_string()))?
        };

        // Get all markdown files in workspace
        let markdown_files = self
            .file_storage
            .glob("**/*.md", &workspace.folder_path)
            .await?;

        // Get existing notes for this workspace
        let existing_notes = self
            .note_repository
            .find_all(NoteFindOptions {
                workspace_id: Some(workspace.id.clone()),
                ..Default::default()
            })
            .await?;

        let mut existing_paths_map: HashMap<String, Note> = HashMap::new();
        for note in &existing_notes {
            if let Some(ref path) = note.file_path {
                existing_paths_map.insert(path.clone(), note.clone());
            }
        }

        let mut found_paths = std::collections::HashSet::new();
        let mut created = 0;
        let mut updated = 0;
        let mut deleted = 0;

        // Process each file
        for relative_path in markdown_files {
            found_paths.insert(relative_path.clone());
            let absolute_path = Path::new(&workspace.folder_path).join(&relative_path);
            let path_str = absolute_path.to_string_lossy().to_string();

            let file_info = self.file_storage.get_file_info(&path_str).await?;
            if file_info.is_none() {
                continue;
            }
            let file_info = file_info.unwrap();

            let existing_note = existing_paths_map.get(&relative_path);

            if existing_note.is_none() {
                // Create new note entry
                let file_content = self.file_storage.read(&path_str).await?;

                // Extract title from content or derive from filename
                let title = if let Some(ref content) = file_content {
                    // ❌ SYNC - NO AWAIT!
                    self.markdown_processor.extract_title(content)?
                } else {
                    None
                };

                let title = title.unwrap_or_else(|| {
                    Path::new(&relative_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled")
                        .to_string()
                });

                let mut note = Note::new(&title, Some(workspace.id.clone()))?;
                note.set_file_path(Some(relative_path.clone()))?;

                self.note_repository.save(&note).await?;

                // Publish event
                if let Some(ref publisher) = self.event_publisher {
                    publisher.emit("note:created", serde_json::json!({"id": note.id}));
                }

                created += 1;
            } else {
                let existing = existing_note.unwrap();
                if existing.updated_at < file_info.modified_at {
                    // Note was modified externally - update timestamp
                    let mut note_entity = existing.clone();

                    // Re-extract title in case it changed
                    let file_content = self.file_storage.read(&path_str).await?;
                    if let Some(content) = file_content {
                        // ❌ SYNC - NO AWAIT!
                        if let Some(new_title) = self.markdown_processor.extract_title(&content)? {
                            if new_title != existing.title {
                                note_entity.update_title(new_title)?;
                            }
                        }
                    }

                    // Force update timestamp by re-setting file path
                    if let Some(ref fp) = existing.file_path {
                        note_entity.set_file_path(Some(fp.clone()))?;
                    }

                    self.note_repository.save(&note_entity).await?;

                    // Publish event
                    if let Some(ref publisher) = self.event_publisher {
                        publisher.emit("note:updated", serde_json::json!({"id": existing.id}));
                    }

                    updated += 1;
                }
            }
        }

        // Mark deleted notes (soft delete)
        for note in existing_notes {
            if let Some(ref file_path) = note.file_path {
                if !found_paths.contains(file_path) && !note.is_deleted {
                    let mut note_entity = note;
                    note_entity.delete();

                    self.note_repository.save(&note_entity).await?;

                    // Publish event
                    if let Some(ref publisher) = self.event_publisher {
                        publisher.emit("note:deleted", serde_json::json!({"id": note_entity.id}));
                    }

                    deleted += 1;
                }
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(SyncWorkspaceResponse {
            notes: SyncWorkspaceStats {
                created,
                updated,
                deleted,
            },
            duration_ms,
        })
    }

    /// Create a folder within a workspace
    async fn create_folder(
        &self,
        request: CreateFolderRequest,
    ) -> DomainResult<CreateFolderResponse> {
        let active_workspace = self
            .workspace_repository
            .find_active()
            .await?
            .ok_or_else(|| DomainError::ValidationError("No active workspace".to_string()))?;

        let base_path = if let Some(parent_path) = request.parent_path {
            Path::new(&active_workspace.folder_path)
                .join(&parent_path)
                .to_string_lossy()
                .to_string()
        } else {
            active_workspace.folder_path.clone()
        };

        let folder_path = Path::new(&base_path)
            .join(&request.name)
            .to_string_lossy()
            .to_string();

        self.file_storage.create_directory(&folder_path).await?;

        // Return relative path from workspace root
        let relative_path = Path::new(&folder_path)
            .strip_prefix(&active_workspace.folder_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(request.name);

        Ok(CreateFolderResponse {
            path: relative_path,
        })
    }

    /// Rename a folder
    async fn rename_folder(
        &self,
        request: RenameFolderRequest,
    ) -> DomainResult<RenameFolderResponse> {
        let active_workspace = self
            .workspace_repository
            .find_active()
            .await?
            .ok_or_else(|| DomainError::ValidationError("No active workspace".to_string()))?;

        if request.name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Folder name is required".to_string(),
            ));
        }

        let absolute_path = Path::new(&active_workspace.folder_path)
            .join(&request.path)
            .to_string_lossy()
            .to_string();

        if !self.file_storage.exists(&absolute_path).await? {
            return Err(DomainError::ValidationError(format!(
                "Folder does not exist: {}",
                request.path
            )));
        }

        let parent_dir = Path::new(&absolute_path)
            .parent()
            .ok_or_else(|| DomainError::ValidationError("Invalid path".to_string()))?;

        let new_absolute_path = parent_dir
            .join(&request.name)
            .to_string_lossy()
            .to_string();

        self.file_storage
            .rename(&absolute_path, &new_absolute_path)
            .await?;

        let new_relative_path = Path::new(&new_absolute_path)
            .strip_prefix(&active_workspace.folder_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(request.name);

        Ok(RenameFolderResponse {
            old_path: request.path,
            new_path: new_relative_path,
        })
    }

    /// Delete a folder
    async fn delete_folder(&self, path: &str) -> DomainResult<()> {
        let active_workspace = self
            .workspace_repository
            .find_active()
            .await?
            .ok_or_else(|| DomainError::ValidationError("No active workspace".to_string()))?;

        if path.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Folder path is required".to_string(),
            ));
        }

        let absolute_path = Path::new(&active_workspace.folder_path)
            .join(path)
            .to_string_lossy()
            .to_string();

        if !self.file_storage.exists(&absolute_path).await? {
            return Err(DomainError::ValidationError(format!(
                "Folder does not exist: {}",
                path
            )));
        }

        self.file_storage.delete_directory(&absolute_path).await?;

        Ok(())
    }

    /// Move a folder to a different location
    async fn move_folder(
        &self,
        request: MoveFolderRequest,
    ) -> DomainResult<MoveFolderResponse> {
        let active_workspace = self
            .workspace_repository
            .find_active()
            .await?
            .ok_or_else(|| DomainError::ValidationError("No active workspace".to_string()))?;

        if request.source_path.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Source path is required".to_string(),
            ));
        }

        let source_absolute_path = Path::new(&active_workspace.folder_path)
            .join(&request.source_path)
            .to_string_lossy()
            .to_string();

        if !self.file_storage.exists(&source_absolute_path).await? {
            return Err(DomainError::ValidationError(format!(
                "Folder does not exist: {}",
                request.source_path
            )));
        }

        let folder_name = Path::new(&request.source_path)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| DomainError::ValidationError("Invalid path".to_string()))?;

        let dest_parent = if let Some(dest_path) = request.destination_path {
            Path::new(&active_workspace.folder_path)
                .join(&dest_path)
                .to_string_lossy()
                .to_string()
        } else {
            active_workspace.folder_path.clone()
        };

        let dest_absolute_path = Path::new(&dest_parent)
            .join(folder_name)
            .to_string_lossy()
            .to_string();

        // Prevent moving a folder into itself
        if dest_absolute_path.starts_with(&format!("{}/", source_absolute_path)) {
            return Err(DomainError::ValidationError(
                "Cannot move a folder into itself".to_string(),
            ));
        }

        self.file_storage
            .rename(&source_absolute_path, &dest_absolute_path)
            .await?;

        let new_relative_path = Path::new(&dest_absolute_path)
            .strip_prefix(&active_workspace.folder_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(folder_name.to_string());

        Ok(MoveFolderResponse {
            old_path: request.source_path,
            new_path: new_relative_path,
        })
    }
}

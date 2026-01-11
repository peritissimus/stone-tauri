/// Version Use Cases Implementation
///
/// Application layer implementations for version history operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    entities::Version,
    errors::{DomainError, DomainResult},
    ports::{
        inbound::VersionUseCases,
        outbound::{FileStorage, NoteRepository, VersionRepository, WorkspaceRepository},
    },
};

/// Implementation of all Version use cases
pub struct VersionUseCasesImpl {
    note_repository: Arc<dyn NoteRepository>,
    version_repository: Arc<dyn VersionRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
    file_storage: Arc<dyn FileStorage>,
}

impl VersionUseCasesImpl {
    pub fn new(
        note_repository: Arc<dyn NoteRepository>,
        version_repository: Arc<dyn VersionRepository>,
        workspace_repository: Arc<dyn WorkspaceRepository>,
        file_storage: Arc<dyn FileStorage>,
    ) -> Self {
        Self {
            note_repository,
            version_repository,
            workspace_repository,
            file_storage,
        }
    }
}

#[async_trait]
impl VersionUseCases for VersionUseCasesImpl {
    /// Get version history for a note
    async fn get_versions(&self, note_id: &str) -> DomainResult<Vec<Version>> {
        // Verify note exists - ✅ ASYNC
        let _note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Note not found: {}", note_id))
            })?;

        // Get all versions for this note - ✅ ASYNC
        let versions = self.version_repository.find_by_note_id(note_id).await?;

        Ok(versions)
    }

    /// Create a new version snapshot
    async fn create_version(&self, note_id: &str) -> DomainResult<Version> {
        // Get note - ✅ ASYNC
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Note not found: {}", note_id))
            })?;

        // Check note has file path and workspace
        if note.file_path.is_none() || note.workspace_id.is_none() {
            return Err(DomainError::ValidationError(
                "Note has no file path".to_string(),
            ));
        }

        let file_path = note.file_path.as_ref().unwrap();
        let workspace_id = note.workspace_id.as_ref().unwrap();

        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Build absolute path using std::path
        use std::path::Path;
        let absolute_path = Path::new(&workspace.folder_path)
            .join(file_path)
            .to_string_lossy()
            .to_string();

        // Read file content - ✅ ASYNC
        // ⚠️ Returns Option<String> - use unwrap_or_default for empty content
        let content = self
            .file_storage
            .read(&absolute_path)
            .await?
            .unwrap_or_default();

        // Get next version number - ✅ ASYNC
        let next_version_number = self
            .version_repository
            .get_next_version_number(note_id)
            .await?;

        // Create version entity - ❌ SYNC but returns Result
        let version = Version::new(
            note_id,
            note.title.clone(),
            content,
            next_version_number,
        )?;

        // Save version - ✅ ASYNC
        self.version_repository.save(&version).await?;

        Ok(version)
    }

    /// Restore a note to a specific version
    async fn restore_version(&self, note_id: &str, version_id: &str) -> DomainResult<()> {
        // Get note - ✅ ASYNC
        let mut note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Note not found: {}", note_id))
            })?;

        // Get version - ✅ ASYNC
        let version = self
            .version_repository
            .find_by_id(version_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Version not found: {}", version_id))
            })?;

        // Validate version belongs to note
        if version.note_id != note_id {
            return Err(DomainError::ValidationError(format!(
                "Version {} does not belong to note {}",
                version_id, note_id
            )));
        }

        // Check note has file path and workspace
        if note.file_path.is_none() || note.workspace_id.is_none() {
            return Err(DomainError::ValidationError(
                "Note has no file path".to_string(),
            ));
        }

        let file_path = note.file_path.as_ref().unwrap();
        let workspace_id = note.workspace_id.as_ref().unwrap();

        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Build absolute path using std::path
        use std::path::Path;
        let absolute_path = Path::new(&workspace.folder_path)
            .join(file_path)
            .to_string_lossy()
            .to_string();

        // Write version content to file - ✅ ASYNC
        self.file_storage
            .write(&absolute_path, &version.content)
            .await?;

        // Update note title - ❌ SYNC but returns Result
        note.update_title(version.title.clone())?;

        // Save note - ✅ ASYNC
        self.note_repository.save(&note).await?;

        Ok(())
    }

    /// Get a specific version
    async fn get_version(&self, version_id: &str) -> DomainResult<Option<Version>> {
        // Simple lookup - ✅ ASYNC
        self.version_repository.find_by_id(version_id).await
    }
}

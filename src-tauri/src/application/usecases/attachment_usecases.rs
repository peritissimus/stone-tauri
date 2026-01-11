/// Attachment Use Cases Implementation
///
/// Application layer implementations for attachment operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    entities::Attachment,
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{
            AddAttachmentRequest, AttachmentUseCases, UploadImageRequest, UploadImageResponse,
        },
        outbound::{AttachmentRepository, FileStorage, NoteRepository, WorkspaceRepository},
    },
};

/// Helper function to get MIME type from file extension
fn get_mime_type(extension: &str) -> String {
    match extension.to_lowercase().as_str() {
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        ".gif" => "image/gif",
        ".webp" => "image/webp",
        ".svg" => "image/svg+xml",
        ".pdf" => "application/pdf",
        ".doc" => "application/msword",
        ".docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ".txt" => "text/plain",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Implementation of all Attachment use cases
pub struct AttachmentUseCasesImpl {
    note_repository: Arc<dyn NoteRepository>,
    attachment_repository: Arc<dyn AttachmentRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
    file_storage: Arc<dyn FileStorage>,
}

impl AttachmentUseCasesImpl {
    pub fn new(
        note_repository: Arc<dyn NoteRepository>,
        attachment_repository: Arc<dyn AttachmentRepository>,
        workspace_repository: Arc<dyn WorkspaceRepository>,
        file_storage: Arc<dyn FileStorage>,
    ) -> Self {
        Self {
            note_repository,
            attachment_repository,
            workspace_repository,
            file_storage,
        }
    }
}

#[async_trait]
impl AttachmentUseCases for AttachmentUseCasesImpl {
    /// Add an attachment to a note
    async fn add_attachment(&self, request: AddAttachmentRequest) -> DomainResult<Attachment> {
        use std::path::Path;

        // Get note - ✅ ASYNC
        let note = self
            .note_repository
            .find_by_id(&request.note_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Note not found: {}", request.note_id))
            })?;

        // Check note has workspace
        let workspace_id = note.workspace_id.as_ref().ok_or_else(|| {
            DomainError::ValidationError("Note has no workspace".to_string())
        })?;

        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Get original filename
        let original_name = request.filename.clone().unwrap_or_else(|| {
            Path::new(&request.file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("untitled")
                .to_string()
        });

        // Get extension
        let ext = Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        // Generate unique filename
        let unique_filename = format!("{}{}", nanoid::nanoid!(), ext);
        let mime_type = get_mime_type(&ext);

        // Create attachments directory
        let attachments_dir = Path::new(&workspace.folder_path)
            .join(".attachments")
            .join(&request.note_id)
            .to_string_lossy()
            .to_string();

        // ✅ ASYNC
        self.file_storage
            .create_directory(&attachments_dir)
            .await?;

        // Build destination path
        let dest_path = Path::new(&attachments_dir)
            .join(&unique_filename)
            .to_string_lossy()
            .to_string();

        // Copy file - ✅ ASYNC
        self.file_storage
            .copy(&request.file_path, &dest_path)
            .await?;

        // Get file info - ✅ ASYNC
        let file_info = self.file_storage.get_file_info(&dest_path).await?;
        let size = file_info.map(|f| f.size as i64).unwrap_or(0);

        // Get relative path
        let relative_path = Path::new(&dest_path)
            .strip_prefix(&workspace.folder_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| dest_path.clone());

        // Create attachment entity - ❌ SYNC but returns Result
        let attachment = Attachment::new(
            request.note_id.clone(),
            unique_filename,
            mime_type,
            size,
            relative_path,
        )?;

        // Save attachment - ✅ ASYNC
        self.attachment_repository.save(&attachment).await?;

        Ok(attachment)
    }

    /// Delete an attachment
    async fn delete_attachment(&self, id: &str, delete_file: Option<bool>) -> DomainResult<()> {
        use std::path::Path;

        let delete_file = delete_file.unwrap_or(true);

        // Get attachment - ✅ ASYNC
        let attachment = self
            .attachment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Attachment not found: {}", id))
            })?;

        // If deleting physical file
        if delete_file {
            // Get note - ✅ ASYNC
            let note = self
                .note_repository
                .find_by_id(&attachment.note_id)
                .await?;

            if let Some(note) = note {
                if let Some(workspace_id) = note.workspace_id {
                    // Get workspace - ✅ ASYNC
                    let workspace = self
                        .workspace_repository
                        .find_by_id(&workspace_id)
                        .await?;

                    if let Some(workspace) = workspace {
                        let absolute_path = Path::new(&workspace.folder_path)
                            .join(&attachment.path)
                            .to_string_lossy()
                            .to_string();

                        // Delete file - ✅ ASYNC
                        self.file_storage.delete(&absolute_path).await?;
                    }
                }
            }
        }

        // Delete attachment from DB - ✅ ASYNC
        self.attachment_repository.delete(id).await?;

        Ok(())
    }

    /// Get all attachments for a note
    async fn get_attachments(&self, note_id: &str) -> DomainResult<Vec<Attachment>> {
        // Simple lookup - ✅ ASYNC
        self.attachment_repository.find_by_note_id(note_id).await
    }

    /// Upload an image and create attachment
    async fn upload_image(
        &self,
        request: UploadImageRequest,
    ) -> DomainResult<UploadImageResponse> {
        use std::path::Path;

        // Get note - ✅ ASYNC
        let note = self
            .note_repository
            .find_by_id(&request.note_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Note not found: {}", request.note_id))
            })?;

        // Check note has workspace
        let workspace_id = note.workspace_id.as_ref().ok_or_else(|| {
            DomainError::ValidationError("Note has no workspace".to_string())
        })?;

        // Get workspace - ✅ ASYNC
        let workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Workspace not found: {}", workspace_id))
            })?;

        // Get extension from filename or default to .png
        let ext = if let Some(ref filename) = request.filename {
            Path::new(filename)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e))
                .unwrap_or_else(|| ".png".to_string())
        } else {
            ".png".to_string()
        };

        // Create temp path
        let temp_path = Path::new(&workspace.folder_path)
            .join(".temp")
            .join(format!("{}{}", nanoid::nanoid!(), ext))
            .to_string_lossy()
            .to_string();

        // Create temp directory - ✅ ASYNC
        let temp_dir = Path::new(&temp_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".");

        self.file_storage.create_directory(temp_dir).await?;

        // Write base64 image data to temp file - ✅ ASYNC
        self.file_storage
            .write(&temp_path, &request.image_data)
            .await?;

        // Add attachment using existing logic
        let attachment = self
            .add_attachment(AddAttachmentRequest {
                note_id: request.note_id.clone(),
                file_path: temp_path.clone(),
                filename: request.filename.clone(),
            })
            .await?;

        // Clean up temp file - ✅ ASYNC
        self.file_storage.delete(&temp_path).await?;

        // Generate markdown link
        let filename = request
            .filename
            .unwrap_or_else(|| "image".to_string());
        let markdown_link = format!("![{}]({})", filename, attachment.path);

        Ok(UploadImageResponse {
            attachment,
            markdown_link,
        })
    }
}

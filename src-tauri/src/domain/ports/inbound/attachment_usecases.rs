use crate::domain::{entities::Attachment, errors::DomainResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAttachmentRequest {
    pub note_id: String,
    pub file_path: String,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadImageRequest {
    pub note_id: String,
    pub image_data: String, // base64
    pub mime_type: String,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadImageResponse {
    pub attachment: Attachment,
    pub markdown_link: String,
}

/// Attachment Use Cases Port (Inbound)
///
/// Defines the contract for attachment operations.
#[async_trait]
pub trait AttachmentUseCases: Send + Sync {
    /// Add an attachment to a note
    async fn add_attachment(&self, request: AddAttachmentRequest) -> DomainResult<Attachment>;

    /// Delete an attachment
    async fn delete_attachment(&self, id: &str, delete_file: Option<bool>) -> DomainResult<()>;

    /// Get all attachments for a note
    async fn get_attachments(&self, note_id: &str) -> DomainResult<Vec<Attachment>>;

    /// Upload an image and create attachment
    async fn upload_image(&self, request: UploadImageRequest) -> DomainResult<UploadImageResponse>;
}

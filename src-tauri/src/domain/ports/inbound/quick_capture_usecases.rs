use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendToJournalResponse {
    pub note_id: String,
    pub appended: bool,
}

/// Quick Capture Use Cases Port (Inbound)
///
/// Defines the contract for quick capture operations.
#[async_trait]
pub trait QuickCaptureUseCases: Send + Sync {
    /// Append content to today's journal note
    async fn append_to_journal(
        &self,
        content: &str,
        workspace_id: Option<&str>,
    ) -> DomainResult<AppendToJournalResponse>;
}

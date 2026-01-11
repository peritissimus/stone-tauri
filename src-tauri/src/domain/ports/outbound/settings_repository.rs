use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Settings Repository Port (Outbound)
///
/// Defines the contract for settings persistence operations.
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Get a setting by key
    async fn get(&self, key: &str) -> DomainResult<Option<Setting>>;

    /// Set a setting (create or update)
    async fn set(&self, key: &str, value: &str) -> DomainResult<Setting>;

    /// Get all settings
    async fn get_all(&self) -> DomainResult<Vec<Setting>>;

    /// Delete a setting by key
    async fn delete(&self, key: &str) -> DomainResult<()>;
}

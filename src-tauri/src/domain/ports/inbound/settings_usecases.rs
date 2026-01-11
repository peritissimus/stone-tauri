use crate::domain::{errors::DomainResult, ports::outbound::Setting};
use async_trait::async_trait;

/// Settings Use Cases Port (Inbound)
///
/// Defines the contract for settings operations.
#[async_trait]
pub trait SettingsUseCases: Send + Sync {
    /// Get a setting by key
    async fn get(&self, key: &str) -> DomainResult<Option<String>>;

    /// Set a setting (create or update)
    async fn set(&self, key: &str, value: &str) -> DomainResult<()>;

    /// Get all settings
    async fn get_all(&self) -> DomainResult<Vec<Setting>>;
}

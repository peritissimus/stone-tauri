/// Settings Use Cases Implementation
///
/// Application layer implementations for settings operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    errors::DomainResult,
    ports::{
        inbound::SettingsUseCases,
        outbound::{Setting, SettingsRepository},
    },
};

/// Implementation of all Settings use cases
pub struct SettingsUseCasesImpl {
    settings_repository: Arc<dyn SettingsRepository>,
}

impl SettingsUseCasesImpl {
    pub fn new(settings_repository: Arc<dyn SettingsRepository>) -> Self {
        Self {
            settings_repository,
        }
    }
}

#[async_trait]
impl SettingsUseCases for SettingsUseCasesImpl {
    /// Get a setting by key
    async fn get(&self, key: &str) -> DomainResult<Option<String>> {
        // Get setting - ✅ ASYNC
        let setting = self.settings_repository.get(key).await?;

        // Extract value from Option<Setting>
        Ok(setting.map(|s| s.value))
    }

    /// Set a setting (create or update)
    async fn set(&self, key: &str, value: &str) -> DomainResult<()> {
        // Set setting - ✅ ASYNC
        // Repository returns Setting, but we don't need it
        self.settings_repository.set(key, value).await?;

        Ok(())
    }

    /// Get all settings
    async fn get_all(&self) -> DomainResult<Vec<Setting>> {
        // Simple pass-through - ✅ ASYNC
        self.settings_repository.get_all().await
    }
}

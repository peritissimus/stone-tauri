/// System Use Cases Implementation
///
/// Application layer implementations for system-level operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    errors::DomainResult,
    ports::{
        inbound::{SelectFolderOptions, SystemUseCases},
        outbound::{system_service::FolderPickerOptions, SystemService},
    },
};

/// Implementation of all System use cases
pub struct SystemUseCasesImpl {
    system_service: Arc<dyn SystemService>,
}

impl SystemUseCasesImpl {
    pub fn new(system_service: Arc<dyn SystemService>) -> Self {
        Self { system_service }
    }
}

#[async_trait]
impl SystemUseCases for SystemUseCasesImpl {
    /// Get system fonts
    async fn get_fonts(&self) -> DomainResult<Vec<String>> {
        // Simple pass-through - ✅ ASYNC
        self.system_service.get_fonts().await
    }

    /// Show folder selection dialog
    async fn select_folder(
        &self,
        options: Option<SelectFolderOptions>,
    ) -> DomainResult<Option<String>> {
        // Convert SelectFolderOptions to FolderPickerOptions
        let service_options = options.map(|opts| FolderPickerOptions {
            title: opts.title,
            default_path: opts.default_path,
            button_label: None,
        });

        // ✅ ASYNC
        self.system_service.select_folder(service_options).await
    }

    /// Validate a file path
    async fn validate_path(&self, path: &str) -> DomainResult<bool> {
        // Simple pass-through - ✅ ASYNC
        self.system_service.validate_path(path).await
    }

    /// Open a path in the file explorer
    fn open_in_folder(&self, path: &str) -> DomainResult<()> {
        // ❌ SYNC - NO AWAIT!
        self.system_service.show_in_folder(path)
    }

    /// Open a URL in the default browser
    async fn open_external(&self, url: &str) -> DomainResult<()> {
        // Simple pass-through - ✅ ASYNC
        self.system_service.open_external(url).await
    }
}

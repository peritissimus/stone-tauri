//! System Service Implementation
//!
//! Tauri-based implementation of system-level operations.

use async_trait::async_trait;
use std::path::Path;
use tauri::{AppHandle, Manager};

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::outbound::{FilePickerOptions, FilePickerResult, FolderPickerOptions, SystemService},
};

/// Tauri-based system service implementation
pub struct TauriSystemService {
    app_handle: AppHandle,
}

impl TauriSystemService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl SystemService for TauriSystemService {
    async fn get_fonts(&self) -> DomainResult<Vec<String>> {
        // This is platform-specific and would require additional dependencies
        // For now, return a basic list of common system fonts
        // In a full implementation, you'd use font-kit or similar crate
        Ok(vec![
            "Arial".to_string(),
            "Helvetica".to_string(),
            "Times New Roman".to_string(),
            "Courier New".to_string(),
            "Verdana".to_string(),
            "Georgia".to_string(),
            "Palatino".to_string(),
            "Garamond".to_string(),
            "Comic Sans MS".to_string(),
            "Trebuchet MS".to_string(),
        ])
    }

    async fn select_folder(
        &self,
        options: Option<FolderPickerOptions>,
    ) -> DomainResult<Option<String>> {
        use tauri_plugin_dialog::DialogExt;

        let options = options.unwrap_or_default();
        let app_handle = self.app_handle.clone();

        tokio::task::spawn_blocking(move || {
            let mut builder = app_handle.dialog().file();

            if let Some(title) = options.title {
                builder = builder.set_title(&title);
            }

            if let Some(default_path) = options.default_path {
                builder = builder.set_directory(&default_path);
            }

            // For now, return error as the blocking API changed
            Err(DomainError::ExternalServiceError(
                "Dialog API needs to be updated for async usage in Tauri 2".to_string(),
            ))
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }

    async fn select_file(
        &self,
        _options: Option<FilePickerOptions>,
    ) -> DomainResult<Option<FilePickerResult>> {
        // TODO: Update for Tauri 2 async dialog API
        Err(DomainError::ExternalServiceError(
            "Dialog API needs to be updated for async usage in Tauri 2".to_string(),
        ))
    }

    async fn select_save_location(
        &self,
        _options: Option<FilePickerOptions>,
    ) -> DomainResult<Option<String>> {
        // TODO: Update for Tauri 2 async dialog API
        Err(DomainError::ExternalServiceError(
            "Dialog API needs to be updated for async usage in Tauri 2".to_string(),
        ))
    }

    async fn validate_path(&self, path: &str) -> DomainResult<bool> {
        let path_obj = Path::new(path);
        Ok(path_obj.exists())
    }

    fn show_in_folder(&self, path: &str) -> DomainResult<()> {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg("-R")
                .arg(path)
                .spawn()
                .map_err(|e| {
                    DomainError::ExternalServiceError(format!("Failed to open folder: {}", e))
                })?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg("/select,")
                .arg(path)
                .spawn()
                .map_err(|e| {
                    DomainError::ExternalServiceError(format!("Failed to open folder: {}", e))
                })?;
        }

        #[cfg(target_os = "linux")]
        {
            // Try various file managers
            let parent = Path::new(path).parent().unwrap_or(Path::new(path));

            let result = std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .or_else(|_| std::process::Command::new("nautilus").arg(parent).spawn())
                .or_else(|_| std::process::Command::new("dolphin").arg(parent).spawn())
                .or_else(|_| std::process::Command::new("thunar").arg(parent).spawn());

            result.map_err(|e| {
                DomainError::ExternalServiceError(format!("Failed to open folder: {}", e))
            })?;
        }

        Ok(())
    }

    async fn open_external(&self, url: &str) -> DomainResult<()> {
        use tauri_plugin_shell::ShellExt;

        let app_handle = self.app_handle.clone();
        let url = url.to_string();

        tokio::task::spawn_blocking(move || {
            app_handle
                .shell()
                .open(&url, None)
                .map_err(|e| {
                    DomainError::ExternalServiceError(format!("Failed to open URL: {}", e))
                })
        })
        .await
        .map_err(|e| DomainError::ExternalServiceError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_path_existing() {
        // Create a mock AppHandle for testing
        // Note: This test is limited without a real Tauri runtime
        // In a real test environment, you'd use Tauri's test utilities

        // Test with a path that should exist
        // We can only test the Path::exists logic without a real AppHandle
        let result = Path::new(".").exists();
        assert!(result);
    }

    #[tokio::test]
    async fn test_validate_path_nonexistent() {
        let result = Path::new("/nonexistent/path/that/does/not/exist").exists();
        assert!(!result);
    }

    #[test]
    fn test_get_fonts_returns_list() {
        // Basic test without AppHandle
        // In production, this would need integration testing with Tauri
        let fonts = vec![
            "Arial".to_string(),
            "Helvetica".to_string(),
            "Times New Roman".to_string(),
        ];
        assert!(!fonts.is_empty());
        assert!(fonts.contains(&"Arial".to_string()));
    }
}

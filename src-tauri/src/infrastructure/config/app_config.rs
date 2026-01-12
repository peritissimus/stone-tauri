//! Application Configuration
//!
//! Top-level application configuration that combines all config modules.

use crate::domain::errors::{DomainError, DomainResult};
use super::{AppPaths, DatabaseConfig};

/// Application environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Development environment
    Development,
    /// Production environment
    Production,
}

impl Environment {
    /// Detect environment from environment variable
    pub fn detect() -> Self {
        match std::env::var("STONE_ENV")
            .or_else(|_| std::env::var("TAURI_ENV"))
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            _ => Environment::Development,
        }
    }

    /// Check if development environment
    pub fn is_dev(&self) -> bool {
        matches!(self, Environment::Development)
    }

    /// Check if production environment
    pub fn is_prod(&self) -> bool {
        matches!(self, Environment::Production)
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Application environment
    pub environment: Environment,

    /// Application paths
    pub paths: AppPaths,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Application name
    pub app_name: String,

    /// Application version
    pub app_version: String,

    /// Enable debug logging
    pub debug_logging: bool,

    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval_secs: u64,
}

impl AppConfig {
    /// Load application configuration
    pub fn load() -> DomainResult<Self> {
        let environment = Environment::detect();
        let paths = AppPaths::new()?;

        // Create database config based on environment
        let database = match environment {
            Environment::Development => {
                DatabaseConfig::development(paths.database_file.clone())
            }
            Environment::Production => {
                DatabaseConfig::production(paths.database_file.clone())
            }
        };

        let debug_logging = environment.is_dev();

        Ok(Self {
            environment,
            paths,
            database,
            app_name: "Stone".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            debug_logging,
            auto_save_interval_secs: 30,
        })
    }

    /// Create configuration with custom paths (useful for testing)
    pub fn with_paths(paths: AppPaths) -> DomainResult<Self> {
        let environment = Environment::detect();
        let database = DatabaseConfig::new(paths.database_file.clone());

        Ok(Self {
            environment,
            paths,
            database,
            app_name: "Stone".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            debug_logging: environment.is_dev(),
            auto_save_interval_secs: 30,
        })
    }

    /// Check if development environment
    pub fn is_dev(&self) -> bool {
        self.environment.is_dev()
    }

    /// Check if production environment
    pub fn is_prod(&self) -> bool {
        self.environment.is_prod()
    }

    /// Validate configuration
    pub fn validate(&self) -> DomainResult<()> {
        // Ensure paths are valid
        if self.paths.app_data_dir.to_str().is_none() {
            return Err(DomainError::ConfigurationError(
                "App data directory path is invalid".to_string(),
            ));
        }

        if self.paths.database_file.to_str().is_none() {
            return Err(DomainError::ConfigurationError(
                "Database file path is invalid".to_string(),
            ));
        }

        // Ensure database config is valid
        if self.database.max_connections == 0 {
            return Err(DomainError::ConfigurationError(
                "Database max_connections must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::load().expect("Failed to load default AppConfig")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_environment_detect() {
        // Default should be Development
        let env = Environment::detect();
        assert!(env.is_dev());
    }

    #[test]
    fn test_environment_methods() {
        let dev = Environment::Development;
        assert!(dev.is_dev());
        assert!(!dev.is_prod());

        let prod = Environment::Production;
        assert!(prod.is_prod());
        assert!(!prod.is_dev());
    }

    #[test]
    fn test_app_config_load() {
        let config = AppConfig::load();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.app_name, "Stone");
        assert!(!config.app_version.is_empty());
    }

    #[test]
    fn test_app_config_with_paths() {
        let temp_dir = TempDir::new().unwrap();
        let paths = AppPaths::with_base_dir(temp_dir.path().to_path_buf());

        let config = AppConfig::with_paths(paths);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.paths.app_data_dir, temp_dir.path());
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig::load().unwrap();
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_environment_specific_config() {
        std::env::set_var("STONE_ENV", "production");
        let env = Environment::detect();
        assert!(env.is_prod());
        std::env::remove_var("STONE_ENV");

        std::env::set_var("STONE_ENV", "development");
        let env = Environment::detect();
        assert!(env.is_dev());
        std::env::remove_var("STONE_ENV");
    }
}

use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatus {
    pub path: String,
    pub size: u64,
    pub is_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheckResult {
    pub ok: bool,
    pub errors: Vec<String>,
}

/// Database Use Cases Port (Inbound)
///
/// Defines the contract for database maintenance operations.
#[async_trait]
pub trait DatabaseUseCases: Send + Sync {
    /// Get database status
    async fn get_status(&self) -> DomainResult<DatabaseStatus>;

    /// Vacuum (compact) the database
    async fn vacuum(&self) -> DomainResult<()>;

    /// Check database integrity
    async fn check_integrity(&self) -> DomainResult<IntegrityCheckResult>;
}

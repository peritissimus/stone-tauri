use crate::domain::{
    errors::DomainResult,
    ports::inbound::{DatabaseStatus, IntegrityCheckResult},
};
use async_trait::async_trait;

/// Database Service Port (Outbound)
///
/// Defines the contract for database maintenance operations.
#[async_trait]
pub trait DatabaseService: Send + Sync {
    /// Get database status (path, size, connection state)
    async fn get_status(&self) -> DomainResult<DatabaseStatus>;

    /// Vacuum (compact) the database to reclaim space
    async fn vacuum(&self) -> DomainResult<()>;

    /// Check database integrity and return any errors
    async fn check_integrity(&self) -> DomainResult<IntegrityCheckResult>;
}

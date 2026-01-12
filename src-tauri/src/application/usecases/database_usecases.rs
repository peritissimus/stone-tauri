/// Database Use Cases Implementation
///
/// Application layer implementations for database maintenance operations.
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    errors::DomainResult,
    ports::{
        inbound::{DatabaseStatus, DatabaseUseCases, IntegrityCheckResult},
        outbound::DatabaseService,
    },
};

/// Implementation of all Database use cases
pub struct DatabaseUseCasesImpl {
    database_service: Arc<dyn DatabaseService>,
}

impl DatabaseUseCasesImpl {
    pub fn new(database_service: Arc<dyn DatabaseService>) -> Self {
        Self { database_service }
    }
}

#[async_trait]
impl DatabaseUseCases for DatabaseUseCasesImpl {
    /// Get database status
    async fn get_status(&self) -> DomainResult<DatabaseStatus> {
        // Simple pass-through - ✅ ASYNC
        self.database_service.get_status().await
    }

    /// Vacuum (compact) the database
    async fn vacuum(&self) -> DomainResult<()> {
        // Simple pass-through - ✅ ASYNC
        self.database_service.vacuum().await
    }

    /// Check database integrity
    async fn check_integrity(&self) -> DomainResult<IntegrityCheckResult> {
        // Simple pass-through - ✅ ASYNC
        self.database_service.check_integrity().await
    }
}

//! Service Adapters (Outbound Ports - External Services)
//!
//! Implementations of service adapters for external systems:
//! - TokioFileStorage: Async file I/O operations using Tokio
//! - PulldownMarkdownService: Markdown parsing and conversion
//! - DieselDatabaseService: Database maintenance operations
//! - TauriSystemService: OS-level operations (dialogs, fonts, etc.)
//! - Git2Service: Git operations via git2/libgit2
//! - TokioEventPublisher: Event publishing using broadcast channels
//! - NotifyFileWatcher: File system watching using notify crate
//! - StubSearchService: Search engine (stub implementation)
//! - StubEmbeddingService: ML embedding generation (stub implementation)
//! - StubExportService: PDF/HTML export (stub implementation)

// Core Services (Phase 1)
pub mod file_storage_impl;
pub mod markdown_service;
pub mod database_service_impl;

// Integration Services (Phase 2)
pub mod system_service_impl;
pub mod git_service_impl;
pub mod event_publisher_impl;

// Advanced Services (Phase 3)
pub mod file_watcher_impl;
pub mod search_service;
pub mod embedding_service_impl;
pub mod export_service_impl;

// Re-exports for convenience
pub use file_storage_impl::TokioFileStorage;
pub use markdown_service::PulldownMarkdownService;
pub use database_service_impl::DieselDatabaseService;
pub use system_service_impl::TauriSystemService;
pub use git_service_impl::Git2Service;
pub use event_publisher_impl::TokioEventPublisher;
pub use file_watcher_impl::NotifyFileWatcher;
pub use search_service::StubSearchService;
pub use embedding_service_impl::StubEmbeddingService;
pub use export_service_impl::StubExportService;

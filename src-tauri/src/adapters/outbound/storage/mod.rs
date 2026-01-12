//! Storage Adapters (Outbound Ports - File Storage)
//!
//! File system storage implementation:
//! - FileStorage: Async file I/O operations using tokio

pub mod file_storage_impl;

// Re-exports
pub use file_storage_impl::TokioFileStorage;

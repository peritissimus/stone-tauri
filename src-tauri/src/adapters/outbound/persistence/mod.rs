//! Persistence Adapters (Outbound Ports - Repositories)
//!
//! Database repository implementations using Diesel ORM.
//! Implements repository traits from domain::ports::outbound.

pub mod db_pool;
pub mod utils;
pub mod mappers;

// Repository implementations
pub mod workspace_repository;
pub mod note_repository;
pub mod notebook_repository;
pub mod tag_repository;
pub mod topic_repository;
pub mod attachment_repository;
pub mod note_link_repository;
pub mod version_repository;
pub mod settings_repository;

// Re-exports
pub use db_pool::{create_pool, get_connection, DbPool, DbConnection};
pub use utils::*;
pub use mappers::*;
pub use workspace_repository::DieselWorkspaceRepository;
pub use note_repository::DieselNoteRepository;
pub use notebook_repository::DieselNotebookRepository;
pub use tag_repository::DieselTagRepository;
pub use topic_repository::DieselTopicRepository;
pub use attachment_repository::DieselAttachmentRepository;
pub use note_link_repository::DieselNoteLinkRepository;
pub use version_repository::DieselVersionRepository;
pub use settings_repository::DieselSettingsRepository;

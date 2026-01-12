//! Entity Mappers
//!
//! Converts between database schema structs and domain entities.

pub mod workspace_mapper;
pub mod note_mapper;
pub mod notebook_mapper;
pub mod tag_mapper;
pub mod topic_mapper;
pub mod attachment_mapper;
pub mod note_link_mapper;
pub mod version_mapper;
pub mod settings_mapper;

pub use workspace_mapper::*;
pub use note_mapper::*;
pub use notebook_mapper::*;
pub use tag_mapper::*;
pub use topic_mapper::*;
pub use attachment_mapper::*;
pub use note_link_mapper::*;
pub use version_mapper::*;
pub use settings_mapper::*;

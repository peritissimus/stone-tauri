//! Minimal NSPanel implementation for macOS
//!
//! This module provides a lightweight NSPanel wrapper specifically for
//! the quick capture floating window. It replaces the external tauri-nspanel
//! dependency with just the functionality we need.
//!
//! Architecture: Infrastructure layer (platform-specific implementation)

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::*;

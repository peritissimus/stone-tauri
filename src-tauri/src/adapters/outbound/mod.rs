/// Outbound Adapters
///
/// Implementations of outbound ports (external dependencies).
/// These adapters implement domain port traits and connect to external systems.
///
/// Structure matches TypeScript implementation:
/// - persistence: Database repositories (Diesel/SQLite)
/// - services: Service adapters (Git, Markdown, Search, etc.)
/// - external: External system adapters (Event publishing, File watching)
/// - storage: File system storage adapters

pub mod external;
pub mod persistence;
pub mod services;
pub mod storage;

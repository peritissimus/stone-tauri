/// Adapters Module
///
/// All adapter implementations following hexagonal architecture.
/// Adapters connect the domain/application core to external systems.
///
/// Structure:
/// - outbound: Outbound adapters (repositories, services, external systems, storage)
///   - persistence: Database repositories
///   - services: Service adapters (Git, Markdown, Search, etc.)
///   - external: External system adapters (Events, File watching)
///   - storage: File storage adapters
/// - inbound: Inbound adapters (Tauri commands, IPC layer)

pub mod inbound;
pub mod outbound;

// Re-exports for convenience
pub use outbound::persistence::*;

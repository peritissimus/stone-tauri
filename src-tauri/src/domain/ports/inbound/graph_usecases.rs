use crate::domain::errors::DomainResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteLinkInfo {
    pub source_id: String,
    pub source_title: String,
    pub target_id: String,
    pub target_title: String,
    pub link_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub val: i32,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphDataOptions {
    pub center_note_id: Option<String>,
    pub depth: Option<i32>,
    pub include_orphans: Option<bool>,
}

/// Graph Use Cases Port (Inbound)
///
/// Defines the contract for link/graph operations.
#[async_trait]
pub trait GraphUseCases: Send + Sync {
    /// Get backlinks (notes that link TO this note)
    async fn get_backlinks(&self, note_id: &str) -> DomainResult<Vec<NoteLinkInfo>>;

    /// Get forward links (notes that this note links TO)
    async fn get_forward_links(&self, note_id: &str) -> DomainResult<Vec<NoteLinkInfo>>;

    /// Get graph data for visualization
    async fn get_graph_data(
        &self,
        options: Option<GraphDataOptions>,
    ) -> DomainResult<GraphData>;

    /// Update links for a note (called after save)
    async fn update_note_links(&self, note_id: &str, content: &str) -> DomainResult<()>;
}

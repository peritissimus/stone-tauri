use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSyncOperation {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTopicClassification {
    pub topic_id: String,
    pub topic_name: String,
    pub confidence: f32,
}

/// Domain events for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DomainEvent {
    #[serde(rename = "note:created")]
    NoteCreated {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        title: String,
        workspace_id: Option<String>,
        notebook_id: Option<String>,
        file_path: Option<String>,
    },
    #[serde(rename = "note:updated")]
    NoteUpdated {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        title: String,
        changes: Vec<String>,
    },
    #[serde(rename = "note:deleted")]
    NoteDeleted {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        title: String,
        permanent: bool,
    },
    #[serde(rename = "note:moved")]
    NoteMoved {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        from_notebook_id: Option<String>,
        to_notebook_id: Option<String>,
    },
    #[serde(rename = "notebook:created")]
    NotebookCreated {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        name: String,
        workspace_id: Option<String>,
        parent_id: Option<String>,
    },
    #[serde(rename = "notebook:updated")]
    NotebookUpdated {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        name: String,
        changes: Vec<String>,
    },
    #[serde(rename = "notebook:deleted")]
    NotebookDeleted {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        name: String,
    },
    #[serde(rename = "tag:created")]
    TagCreated {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        name: String,
    },
    #[serde(rename = "tag:deleted")]
    TagDeleted {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        name: String,
    },
    #[serde(rename = "note:tagged")]
    NoteTagged {
        timestamp: chrono::DateTime<chrono::Utc>,
        note_id: String,
        tag_id: String,
        tag_name: String,
    },
    #[serde(rename = "note:untagged")]
    NoteUntagged {
        timestamp: chrono::DateTime<chrono::Utc>,
        note_id: String,
        tag_id: String,
        tag_name: String,
    },
    #[serde(rename = "workspace:activated")]
    WorkspaceActivated {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        name: String,
        folder_path: String,
    },
    #[serde(rename = "file:synced")]
    FileSynced {
        timestamp: chrono::DateTime<chrono::Utc>,
        file_path: String,
        operation: FileSyncOperation,
    },
    #[serde(rename = "topic:created")]
    TopicCreated {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        name: String,
    },
    #[serde(rename = "topic:updated")]
    TopicUpdated {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
        name: String,
    },
    #[serde(rename = "topic:deleted")]
    TopicDeleted {
        timestamp: chrono::DateTime<chrono::Utc>,
        id: String,
    },
    #[serde(rename = "note:classified")]
    NoteClassified {
        timestamp: chrono::DateTime<chrono::Utc>,
        note_id: String,
        topics: Vec<EventTopicClassification>,
    },
    #[serde(rename = "embedding:progress")]
    EmbeddingProgress {
        timestamp: chrono::DateTime<chrono::Utc>,
        processed: i32,
        total: i32,
        failed: i32,
    },
    #[serde(rename = "db:vacuum:progress")]
    DbVacuumProgress {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    #[serde(rename = "db:vacuum:complete")]
    DbVacuumComplete {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::NoteCreated { .. } => "note:created",
            DomainEvent::NoteUpdated { .. } => "note:updated",
            DomainEvent::NoteDeleted { .. } => "note:deleted",
            DomainEvent::NoteMoved { .. } => "note:moved",
            DomainEvent::NotebookCreated { .. } => "notebook:created",
            DomainEvent::NotebookUpdated { .. } => "notebook:updated",
            DomainEvent::NotebookDeleted { .. } => "notebook:deleted",
            DomainEvent::TagCreated { .. } => "tag:created",
            DomainEvent::TagDeleted { .. } => "tag:deleted",
            DomainEvent::NoteTagged { .. } => "note:tagged",
            DomainEvent::NoteUntagged { .. } => "note:untagged",
            DomainEvent::WorkspaceActivated { .. } => "workspace:activated",
            DomainEvent::FileSynced { .. } => "file:synced",
            DomainEvent::TopicCreated { .. } => "topic:created",
            DomainEvent::TopicUpdated { .. } => "topic:updated",
            DomainEvent::TopicDeleted { .. } => "topic:deleted",
            DomainEvent::NoteClassified { .. } => "note:classified",
            DomainEvent::EmbeddingProgress { .. } => "embedding:progress",
            DomainEvent::DbVacuumProgress { .. } => "db:vacuum:progress",
            DomainEvent::DbVacuumComplete { .. } => "db:vacuum:complete",
        }
    }
}

pub type EventHandler = Arc<dyn Fn(DomainEvent) + Send + Sync>;

/// Event Publisher Port (Outbound)
///
/// Defines the contract for publishing domain events.
/// Implementations can be in-process event bus, message queue, etc.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a domain event
    fn publish(&self, event: DomainEvent);

    /// Publish multiple events
    fn publish_all(&self, events: Vec<DomainEvent>);

    /// Emit a simple event using channel name
    fn emit(&self, channel: &str, payload: serde_json::Value);

    /// Subscribe to events of a specific type
    fn subscribe(&self, event_type: &str, handler: EventHandler) -> Box<dyn Fn() + Send + Sync>;

    /// Subscribe to all events
    fn subscribe_all(&self, handler: EventHandler) -> Box<dyn Fn() + Send + Sync>;
}

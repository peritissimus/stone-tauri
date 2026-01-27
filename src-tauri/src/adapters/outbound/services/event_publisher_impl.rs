//! Event Publisher Implementation
//!
//! In-process event bus implementation using Tokio broadcast channels and
//! Tauri event system for frontend communication.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

use crate::domain::ports::outbound::{DomainEvent, EventHandler, EventPublisher};

/// In-process event publisher using broadcast channels
pub struct TokioEventPublisher {
    /// Global broadcast channel for all events
    all_events_tx: broadcast::Sender<DomainEvent>,
    /// Per-event-type broadcast channels
    type_channels: Arc<Mutex<HashMap<String, broadcast::Sender<DomainEvent>>>>,
    /// Tauri app handle for emitting to frontend
    app_handle: Option<AppHandle>,
}

impl TokioEventPublisher {
    /// Create a new event publisher without Tauri integration
    pub fn new() -> Self {
        let (all_events_tx, _) = broadcast::channel(1000);

        Self {
            all_events_tx,
            type_channels: Arc::new(Mutex::new(HashMap::new())),
            app_handle: None,
        }
    }

    /// Create a new event publisher with Tauri integration
    pub fn with_tauri(app_handle: AppHandle) -> Self {
        let (all_events_tx, _) = broadcast::channel(1000);

        Self {
            all_events_tx,
            type_channels: Arc::new(Mutex::new(HashMap::new())),
            app_handle: Some(app_handle),
        }
    }

    /// Get or create a broadcast channel for a specific event type
    fn get_or_create_channel(&self, event_type: &str) -> broadcast::Sender<DomainEvent> {
        let mut channels = self.type_channels.lock().unwrap();

        channels
            .entry(event_type.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(1000);
                tx
            })
            .clone()
    }

    /// Emit event to Tauri frontend if app_handle is available
    fn emit_to_frontend(&self, event: &DomainEvent) {
        if let Some(app_handle) = &self.app_handle {
            let event_type = event.event_type();

            // Emit to frontend
            if let Err(e) = app_handle.emit(event_type, event) {
                tracing::error!("Failed to emit event '{}' to frontend: {}", event_type, e);
            }

            // Also emit to a general "domain-event" channel
            if let Err(e) = app_handle.emit("domain-event", event) {
                tracing::error!("Failed to emit domain event to frontend: {}", e);
            }
        }
    }
}

impl Default for TokioEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPublisher for TokioEventPublisher {
    fn publish(&self, event: DomainEvent) {
        let event_type = event.event_type().to_string();

        // Publish to global channel
        let _ = self.all_events_tx.send(event.clone());

        // Publish to type-specific channel
        let type_tx = self.get_or_create_channel(&event_type);
        let _ = type_tx.send(event.clone());

        // Emit to frontend if available
        self.emit_to_frontend(&event);
    }

    fn publish_all(&self, events: Vec<DomainEvent>) {
        for event in events {
            self.publish(event);
        }
    }

    fn emit(&self, channel: &str, payload: serde_json::Value) {
        if let Some(app_handle) = &self.app_handle {
            if let Err(e) = app_handle.emit(channel, payload) {
                eprintln!("Failed to emit to channel '{}': {}", channel, e);
            }
        }
    }

    fn subscribe(&self, event_type: &str, handler: EventHandler) -> Box<dyn Fn() + Send + Sync> {
        let type_tx = self.get_or_create_channel(event_type);
        let mut rx = type_tx.subscribe();

        let handle = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                handler(event);
            }
        });

        // Return unsubscribe function
        Box::new(move || {
            handle.abort();
        })
    }

    fn subscribe_all(&self, handler: EventHandler) -> Box<dyn Fn() + Send + Sync> {
        let mut rx = self.all_events_tx.subscribe();

        let handle = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                handler(event);
            }
        });

        // Return unsubscribe function
        Box::new(move || {
            handle.abort();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let publisher = TokioEventPublisher::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        // Subscribe to note:created events
        let _unsubscribe = publisher.subscribe("note:created", Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        // Give the subscriber a moment to set up
        sleep(Duration::from_millis(10)).await;

        // Publish an event
        publisher.publish(DomainEvent::NoteCreated {
            timestamp: Utc::now(),
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            workspace_id: None,
            notebook_id: None,
            file_path: None,
        });

        // Wait for event to be processed
        sleep(Duration::from_millis(50)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_subscribe_all() {
        let publisher = TokioEventPublisher::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        // Subscribe to all events
        let _unsubscribe = publisher.subscribe_all(Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        sleep(Duration::from_millis(10)).await;

        // Publish multiple different events
        publisher.publish(DomainEvent::NoteCreated {
            timestamp: Utc::now(),
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            workspace_id: None,
            notebook_id: None,
            file_path: None,
        });

        publisher.publish(DomainEvent::TagCreated {
            timestamp: Utc::now(),
            id: "tag-1".to_string(),
            name: "Test Tag".to_string(),
        });

        sleep(Duration::from_millis(50)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_publish_all() {
        let publisher = TokioEventPublisher::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _unsubscribe = publisher.subscribe_all(Arc::new(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        sleep(Duration::from_millis(10)).await;

        let events = vec![
            DomainEvent::NoteCreated {
                timestamp: Utc::now(),
                id: "note-1".to_string(),
                title: "Test Note 1".to_string(),
                workspace_id: None,
                notebook_id: None,
                file_path: None,
            },
            DomainEvent::NoteCreated {
                timestamp: Utc::now(),
                id: "note-2".to_string(),
                title: "Test Note 2".to_string(),
                workspace_id: None,
                notebook_id: None,
                file_path: None,
            },
        ];

        publisher.publish_all(events);

        sleep(Duration::from_millis(50)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_type_specific_subscription() {
        let publisher = TokioEventPublisher::new();
        let note_counter = Arc::new(AtomicUsize::new(0));
        let tag_counter = Arc::new(AtomicUsize::new(0));

        let note_counter_clone = note_counter.clone();
        let tag_counter_clone = tag_counter.clone();

        // Subscribe to different event types
        let _unsub1 = publisher.subscribe("note:created", Arc::new(move |_event| {
            note_counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        let _unsub2 = publisher.subscribe("tag:created", Arc::new(move |_event| {
            tag_counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        sleep(Duration::from_millis(10)).await;

        // Publish events
        publisher.publish(DomainEvent::NoteCreated {
            timestamp: Utc::now(),
            id: "note-1".to_string(),
            title: "Test Note".to_string(),
            workspace_id: None,
            notebook_id: None,
            file_path: None,
        });

        publisher.publish(DomainEvent::TagCreated {
            timestamp: Utc::now(),
            id: "tag-1".to_string(),
            name: "Test Tag".to_string(),
        });

        sleep(Duration::from_millis(50)).await;

        // Each subscriber should only receive their specific event type
        assert_eq!(note_counter.load(Ordering::SeqCst), 1);
        assert_eq!(tag_counter.load(Ordering::SeqCst), 1);
    }
}

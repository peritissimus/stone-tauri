/// Graph Use Cases Implementation
///
/// Application layer implementations for graph and link operations.
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::{
    entities::NoteLink,
    errors::{DomainError, DomainResult},
    ports::{
        inbound::{GraphDataOptions, GraphUseCases, NoteLinkInfo, GraphData, GraphNode, GraphLink},
        outbound::{NoteFindOptions, NoteLinkRepository, NoteRepository, WorkspaceRepository},
    },
    services::LinkExtractor,
};

/// Implementation of all Graph use cases
pub struct GraphUseCasesImpl {
    note_repository: Arc<dyn NoteRepository>,
    note_link_repository: Arc<dyn NoteLinkRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
}

impl GraphUseCasesImpl {
    pub fn new(
        note_repository: Arc<dyn NoteRepository>,
        note_link_repository: Arc<dyn NoteLinkRepository>,
        workspace_repository: Arc<dyn WorkspaceRepository>,
    ) -> Self {
        Self {
            note_repository,
            note_link_repository,
            workspace_repository,
        }
    }
}

#[async_trait]
impl GraphUseCases for GraphUseCasesImpl {
    /// Get backlinks (notes that link TO this note)
    async fn get_backlinks(&self, note_id: &str) -> DomainResult<Vec<NoteLinkInfo>> {
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        let source_notes = self.note_link_repository.get_backlinks(note_id).await?;

        let links = source_notes
            .into_iter()
            .map(|source_note| NoteLinkInfo {
                source_id: source_note.id,
                source_title: source_note.title,
                target_id: note_id.to_string(),
                target_title: note.title.clone(),
                link_text: String::new(),
            })
            .collect();

        Ok(links)
    }

    /// Get forward links (notes that this note links TO)
    async fn get_forward_links(&self, note_id: &str) -> DomainResult<Vec<NoteLinkInfo>> {
        let note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        let target_notes = self.note_link_repository.get_forward_links(note_id).await?;

        let links = target_notes
            .into_iter()
            .map(|target_note| NoteLinkInfo {
                source_id: note_id.to_string(),
                source_title: note.title.clone(),
                target_id: target_note.id,
                target_title: target_note.title,
                link_text: String::new(),
            })
            .collect();

        Ok(links)
    }

    /// Get graph data for visualization
    async fn get_graph_data(
        &self,
        options: Option<GraphDataOptions>,
    ) -> DomainResult<GraphData> {
        // Extract options with defaults
        let GraphDataOptions {
            center_note_id,
            depth,
            include_orphans,
        } = options.unwrap_or_default();

        let depth = depth.unwrap_or(2);
        let include_orphans = include_orphans.unwrap_or(false);

        // Get active workspace
        let active_workspace = self.workspace_repository.find_active().await?;
        if active_workspace.is_none() {
            return Ok(GraphData {
                nodes: Vec::new(),
                links: Vec::new(),
            });
        }
        let workspace = active_workspace.unwrap();

        // Get all notes for active workspace only
        let all_notes = self
            .note_repository
            .find_all(NoteFindOptions {
                workspace_id: Some(workspace.id),
                is_deleted: Some(false),
                ..Default::default()
            })
            .await?;

        let all_links = self.note_link_repository.find_all().await?;

        // Build link counts
        let mut link_counts: HashMap<String, i32> = HashMap::new();
        for link in &all_links {
            *link_counts
                .entry(link.source_note_id.clone())
                .or_insert(0) += 1;
            *link_counts
                .entry(link.target_note_id.clone())
                .or_insert(0) += 1;
        }

        // Determine which notes to include
        let mut included_notes: HashSet<String> = HashSet::new();

        if let Some(center_id) = center_note_id {
            // BFS to find notes within depth
            let mut queue: VecDeque<(String, i32)> = VecDeque::new();
            queue.push_back((center_id.clone(), 0));
            included_notes.insert(center_id);

            while let Some((id, d)) = queue.pop_front() {
                if d >= depth {
                    continue;
                }

                // Find all links involving this note
                for link in &all_links {
                    let other_id = if link.source_note_id == id {
                        Some(&link.target_note_id)
                    } else if link.target_note_id == id {
                        Some(&link.source_note_id)
                    } else {
                        None
                    };

                    if let Some(other) = other_id {
                        if !included_notes.contains(other) {
                            included_notes.insert(other.clone());
                            queue.push_back((other.clone(), d + 1));
                        }
                    }
                }
            }
        } else {
            // Include all notes (optionally filter orphans)
            for note in &all_notes {
                let link_count = link_counts.get(&note.id).copied().unwrap_or(0);
                if include_orphans || link_count > 0 {
                    included_notes.insert(note.id.clone());
                }
            }
        }

        // Build nodes
        let nodes: Vec<GraphNode> = all_notes
            .into_iter()
            .filter(|note| included_notes.contains(&note.id))
            .map(|note| {
                let val = link_counts.get(&note.id).copied().unwrap_or(1);
                GraphNode {
                    id: note.id,
                    name: note.title,
                    val,
                    color: None,
                }
            })
            .collect();

        // Build links
        let links: Vec<GraphLink> = all_links
            .into_iter()
            .filter(|link| {
                included_notes.contains(&link.source_note_id)
                    && included_notes.contains(&link.target_note_id)
            })
            .map(|link| GraphLink {
                source: link.source_note_id,
                target: link.target_note_id,
            })
            .collect();

        Ok(GraphData { nodes, links })
    }

    /// Update links for a note (called after content change)
    async fn update_note_links(&self, note_id: &str, content: &str) -> DomainResult<()> {
        let _note = self
            .note_repository
            .find_by_id(note_id)
            .await?
            .ok_or_else(|| DomainError::NoteNotFound(note_id.to_string()))?;

        // Extract referenced note titles from content
        let referenced_titles = LinkExtractor::get_referenced_note_titles(content);

        // Delete existing links from this note
        self.note_link_repository
            .delete_from_note(note_id)
            .await?;

        // Get active workspace
        let active_workspace = self.workspace_repository.find_active().await?;
        if active_workspace.is_none() {
            return Ok(());
        }
        let workspace = active_workspace.unwrap();

        // Find target notes by title and create links
        let all_notes = self
            .note_repository
            .find_all(NoteFindOptions {
                workspace_id: Some(workspace.id),
                is_deleted: Some(false),
                ..Default::default()
            })
            .await?;

        for title in referenced_titles {
            let title_lower = title.to_lowercase();

            // Find target note with case-insensitive match
            let target_note = all_notes.iter().find(|n| {
                n.title.to_lowercase() == title_lower
            });

            if let Some(target) = target_note {
                // Don't create self-referencing links
                if target.id != note_id {
                    let link = NoteLink::new(note_id, target.id.clone())?;
                    self.note_link_repository.save(&link).await?;
                }
            }
        }

        Ok(())
    }
}

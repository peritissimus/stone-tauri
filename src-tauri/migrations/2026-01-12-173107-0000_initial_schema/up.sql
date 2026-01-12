-- Initial Schema Migration
-- Creates all 11 tables for the Stone note-taking application

-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- Workspaces table
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    folder_path TEXT NOT NULL UNIQUE,
    is_active INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    last_accessed_at INTEGER NOT NULL
);

-- Notebooks table (hierarchical structure with parent_id)
CREATE TABLE notebooks (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    workspace_id TEXT,
    folder_path TEXT,
    icon TEXT DEFAULT '📁',
    color TEXT DEFAULT '#3b82f6',
    position INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE INDEX idx_notebooks_workspace_id ON notebooks(workspace_id);
CREATE INDEX idx_notebooks_folder_path ON notebooks(folder_path);
CREATE INDEX idx_notebooks_parent_id ON notebooks(parent_id);

-- Notes table (core entity with embeddings and flags)
CREATE TABLE notes (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT DEFAULT 'Untitled',
    file_path TEXT,
    notebook_id TEXT,
    workspace_id TEXT,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at INTEGER,
    embedding BLOB,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE SET NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE INDEX idx_notes_workspace_id ON notes(workspace_id);
CREATE INDEX idx_notes_notebook_id ON notes(notebook_id);
CREATE INDEX idx_notes_file_path ON notes(file_path);
CREATE INDEX idx_notes_flags ON notes(is_favorite, is_pinned, is_archived, is_deleted);
CREATE INDEX idx_notes_updated_at ON notes(updated_at);
CREATE INDEX idx_notes_created_at ON notes(created_at);
CREATE INDEX idx_notes_deleted ON notes(is_deleted);

-- Tags table
CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    color TEXT DEFAULT '#6b7280',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Note Tags junction table (many-to-many)
CREATE TABLE note_tags (
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (note_id, tag_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_note_tags_note_id ON note_tags(note_id);
CREATE INDEX idx_note_tags_tag_id ON note_tags(tag_id);
CREATE INDEX idx_note_tags_composite ON note_tags(tag_id, note_id);

-- Note Links table (bidirectional note connections)
CREATE TABLE note_links (
    source_note_id TEXT NOT NULL,
    target_note_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (source_note_id, target_note_id),
    FOREIGN KEY (source_note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_note_id) REFERENCES notes(id) ON DELETE CASCADE
);

-- Attachments table
CREATE TABLE attachments (
    id TEXT PRIMARY KEY NOT NULL,
    note_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    path TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

CREATE INDEX idx_attachments_note_id ON attachments(note_id);

-- Note Versions table (version history)
CREATE TABLE note_versions (
    id TEXT PRIMARY KEY NOT NULL,
    note_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

-- Topics table (predefined + auto-discovered topics with ML centroids)
CREATE TABLE topics (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT DEFAULT '#6366f1',
    is_predefined INTEGER NOT NULL DEFAULT 0,
    centroid BLOB,
    note_count INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_topics_name ON topics(name);
CREATE INDEX idx_topics_is_predefined ON topics(is_predefined);

-- Note-Topic junction table (many-to-many with confidence scores)
CREATE TABLE note_topics (
    note_id TEXT NOT NULL,
    topic_id TEXT NOT NULL,
    confidence REAL DEFAULT 1.0,
    is_manual INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (note_id, topic_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
);

CREATE INDEX idx_note_topics_note_id ON note_topics(note_id);
CREATE INDEX idx_note_topics_topic_id ON note_topics(topic_id);

-- Settings table (key-value store for app configuration)
CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

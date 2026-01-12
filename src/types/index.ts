/**
 * Shared Types for Stone Tauri Application
 *
 * These types are used for communication between the Tauri backend and React frontend.
 */

// IDs
export type UUID = string & { readonly __brand: 'UUID' };

// Timestamps
export type UnixTimestamp = number;

// Base Entity Types
export interface Workspace {
  id: string;
  name: string;
  path: string;
  is_active: boolean | number;
  git_enabled: boolean | number;
  git_remote_url: string | null;
  created_at: Date | string;
  updated_at: Date | string;
}

export interface Note {
  id: string;
  title: string;
  path: string;
  notebook_id: string | null;
  is_favorite: boolean | number;
  is_pinned: boolean | number;
  is_archived: boolean | number;
  is_deleted: boolean | number;
  workspace_id: string;
  file_modified_at: Date | string;
  created_at: Date | string;
  updated_at: Date | string;
}

export interface Notebook {
  id: string;
  name: string;
  parent_id: string | null;
  position: number;
  is_expanded: boolean | number;
  created_at: Date | string;
  updated_at: Date | string;
}

export interface NotebookWithCount extends Notebook {
  note_count: number;
}

export interface Tag {
  id: string;
  name: string;
  color: string | null;
  created_at: Date | string;
  updated_at: Date | string;
}

export interface TagWithCount extends Tag {
  note_count: number;
}

export interface Topic {
  id: string;
  name: string;
  color: string | null;
  centroid: Uint8Array | null;
  created_at: Date | string;
  updated_at: Date | string;
}

export interface TopicWithCount extends Topic {
  noteCount: number;
}

export interface Attachment {
  id: string;
  note_id: string;
  file_path: string;
  file_name: string;
  file_size: number;
  mime_type: string | null;
  created_at: Date | string;
}

export interface NoteVersion {
  id: string;
  note_id: string;
  version_number: number;
  title: string;
  content: string;
  created_at: Date | string;
}

// Classification result from embedding similarity
export interface ClassificationResult {
  topicId: string;
  topicName: string;
  confidence: number;
}

// Similar note result from semantic search
export interface SimilarNote {
  noteId: string;
  title: string;
  distance: number;
}

// Embedding status
export interface EmbeddingStatus {
  ready: boolean;
  totalNotes: number;
  embeddedNotes: number;
  pendingNotes: number;
}

// Settings
export interface Settings {
  key: string;
  value: string;
  updated_at: UnixTimestamp;
}

// API Request/Response Types
export interface IpcResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: unknown;
  };
}

// Todo Types
export interface TodoItem {
  id: string;
  noteId: UUID;
  noteTitle: string | null;
  notePath: string | null;
  text: string;
  state: 'todo' | 'doing' | 'waiting' | 'hold' | 'done' | 'canceled' | 'idea';
  checked: boolean;
  createdAt?: Date;
  updatedAt?: Date;
}

// Search Types
export interface SearchResult {
  id: UUID;
  title: string;
  notebookId: UUID | null;
  relevance?: number;
  similarity?: number;
  score?: number;
  title_highlight?: string;
  search_type?: 'fts' | 'semantic' | 'hybrid';
}

export interface SearchResults {
  results: SearchResult[];
  total: number;
  query_time_ms: number;
}

export interface VectorSearchResult extends SearchResult {
  similarity: number;
}

// Backup Types
export interface BackupMetadata {
  version: string;
  timestamp: UnixTimestamp;
  databaseVersion: number;
  noteCount: number;
  databaseSize: number;
  vectorSize: number;
  attachmentSize: number;
  checksum: string;
}

// Database Status
export interface DatabaseStatus {
  version: number;
  isMigrating: boolean;
  noteCount: number;
  notebookCount: number;
  tagCount: number;
  attachmentCount: number;
  databaseSize: number;
  vectorSize: number;
  lastBackup?: UnixTimestamp;
  lastDefrag?: UnixTimestamp;
  integrityOk: boolean;
  error?: string;
}

// Database Operation Responses
export interface BackupResult {
  size: number;
  path: string;
  timestamp: UnixTimestamp;
}

export interface VacuumResult {
  size_before: number;
  size_after: number;
  freed_bytes: number;
}

export interface IntegrityResult {
  ok: boolean;
  foreign_keys_ok: boolean;
  errors: string[];
  warnings: string[];
}

// Error Types
export interface AppError {
  code: string;
  message: string;
  details?: unknown;
  timestamp: UnixTimestamp;
}

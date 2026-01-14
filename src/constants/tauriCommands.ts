/**
 * Tauri Command Constants for Stone Application
 *
 * This file maps the old Electron IPC channel names to Tauri command names.
 * Electron used a pattern like "entity:action" (e.g., "notes:create")
 * Tauri uses snake_case function names (e.g., "create_note")
 */

// Workspace Operations
export const WORKSPACE_COMMANDS = {
  CREATE: 'create_workspace',
  UPDATE: 'update_workspace',
  DELETE: 'delete_workspace',
  GET_ALL: 'list_workspaces',
  GET_ACTIVE: 'get_active_workspace',
  SET_ACTIVE: 'set_active_workspace',
  SCAN: 'scan_workspace',
  SYNC: 'sync_workspace',
  CREATE_FOLDER: 'create_folder',
  RENAME_FOLDER: 'rename_folder',
  DELETE_FOLDER: 'delete_folder',
  MOVE_FOLDER: 'move_folder',
  VALIDATE_PATH: 'validate_path',
  SELECT_FOLDER: 'select_folder',
} as const;

// Note Operations
export const NOTE_COMMANDS = {
  CREATE: 'create_note',
  UPDATE: 'update_note',
  DELETE: 'delete_note',
  GET: 'get_note',
  GET_BY_PATH: 'get_note_by_path',
  GET_CONTENT: 'get_note_content',
  GET_ALL: 'get_all_notes',
  GET_ALL_TODOS: 'get_all_tasks',
  GET_NOTE_TODOS: 'get_note_tasks',
  UPDATE_TASK_STATE: 'update_task_state',
  TOGGLE_TASK: 'toggle_task',
  FAVORITE: 'toggle_favorite',
  PIN: 'toggle_pin',
  ARCHIVE: 'archive_note',
  GET_VERSIONS: 'get_versions',
  GET_VERSION: 'get_version',
  CREATE_VERSION: 'create_version',
  RESTORE_VERSION: 'restore_version',
  GET_BACKLINKS: 'get_backlinks',
  GET_FORWARD_LINKS: 'get_forward_links',
  GET_GRAPH_DATA: 'get_graph_data',
  MOVE: 'move_note',
  EXPORT_HTML: 'export_note_html',
  EXPORT_PDF: 'export_note_pdf',
  EXPORT_MARKDOWN: 'export_note_markdown',
} as const;

// Notebook Operations
export const NOTEBOOK_COMMANDS = {
  CREATE: 'create_notebook',
  UPDATE: 'update_notebook',
  DELETE: 'delete_notebook',
  GET_ALL: 'list_notebooks',
  MOVE: 'move_notebook',
} as const;

// Tag Operations
export const TAG_COMMANDS = {
  CREATE: 'create_tag',
  DELETE: 'delete_tag',
  GET_ALL: 'list_tags',
  ADD_TO_NOTE: 'add_tag_to_note',
  REMOVE_FROM_NOTE: 'remove_tag_from_note',
} as const;

// Topic Operations (Semantic Classification)
export const TOPIC_COMMANDS = {
  GET_ALL: 'list_topics',
  GET_BY_ID: 'get_topic',
  CREATE: 'create_topic',
  UPDATE: 'update_topic',
  DELETE: 'delete_topic',
  GET_NOTES_BY_TOPIC: 'get_notes_for_topic',
  GET_TOPICS_FOR_NOTE: 'get_topics_for_note',
  ASSIGN_TO_NOTE: 'assign_topic_to_note',
  REMOVE_FROM_NOTE: 'remove_topic_from_note',
  CLASSIFY_NOTE: 'classify_note',
  CLASSIFY_ALL: 'classify_all_notes',
  RECLASSIFY_ALL: 'classify_all_notes', // Same as CLASSIFY_ALL
  SEMANTIC_SEARCH: 'semantic_search',
  GET_SIMILAR_NOTES: 'get_similar_notes',
  RECOMPUTE_CENTROIDS: 'classify_all_notes', // Uses classify_all which recomputes
  GET_EMBEDDING_STATUS: 'get_embedding_status',
  INITIALIZE: 'classify_all_notes', // Same as CLASSIFY_ALL
} as const;

// Search Operations
export const SEARCH_COMMANDS = {
  FULL_TEXT: 'search_notes',
  SEMANTIC: 'semantic_search',
  HYBRID: 'hybrid_search',
  BY_TAG: 'search_notes', // Can filter by tag using params
  BY_DATE_RANGE: 'search_by_date_range',
} as const;

// Attachment Operations
export const ATTACHMENT_COMMANDS = {
  ADD: 'add_attachment',
  DELETE: 'delete_attachment',
  GET_ALL: 'get_attachments_for_note',
  UPLOAD_IMAGE: 'upload_image',
} as const;

// Database Operations
export const DATABASE_COMMANDS = {
  GET_STATUS: 'get_database_status',
  RUN_MIGRATIONS: 'run_migrations', // May not be exposed
  BACKUP: 'backup_database', // May not be exposed
  RESTORE: 'restore_database', // May not be exposed
  EXPORT: 'export_database', // May not be exposed
  IMPORT: 'import_database', // May not be exposed
  VACUUM: 'vacuum_database',
  CHECK_INTEGRITY: 'check_database_integrity',
  GET_MIGRATION_HISTORY: 'get_migration_history', // May not be exposed
} as const;

// Settings Operations
export const SETTINGS_COMMANDS = {
  GET: 'get_setting',
  SET: 'set_setting',
  GET_ALL: 'get_all_settings',
} as const;

// System Operations
export const SYSTEM_COMMANDS = {
  GET_FONTS: 'get_system_fonts',
} as const;

// Git Operations
export const GIT_COMMANDS = {
  GET_STATUS: 'get_git_status',
  INIT: 'git_init',
  COMMIT: 'git_commit',
  PULL: 'git_pull', // May not be exposed yet
  PUSH: 'git_push', // May not be exposed yet
  SYNC: 'git_sync',
  SET_REMOTE: 'git_set_remote',
  GET_COMMITS: 'git_get_history',
} as const;

// Performance Operations
export const PERFORMANCE_COMMANDS = {
  GET_SNAPSHOT: 'get_performance_snapshot',
  GET_MEMORY: 'get_memory_metrics',
  GET_CPU: 'get_cpu_metrics',
  GET_IPC_STATS: 'get_ipc_stats',
  GET_DB_STATS: 'get_db_stats',
  GET_STARTUP: 'get_startup_metrics',
  CLEAR_HISTORY: 'clear_performance_history',
} as const;

// Quick Capture Operations
export const QUICK_CAPTURE_COMMANDS = {
  APPEND_TO_JOURNAL: 'append_to_journal',
} as const;

// Helper function to get command name from old channel name
export function getCommandFromChannel(channel: string): string {
  // This is a runtime helper for dynamic channel->command conversion if needed
  const [entity, action] = channel.split(':');

  // Convert to snake_case and handle special cases
  const snakeCaseAction = action.replace(/([A-Z])/g, '_$1').toLowerCase();
  return `${snakeCaseAction}_${entity}`;
}

// Event names (Tauri uses kebab-case for events)
export const EVENTS = {
  // Workspace events
  WORKSPACE_CREATED: 'workspace-created',
  WORKSPACE_UPDATED: 'workspace-updated',
  WORKSPACE_DELETED: 'workspace-deleted',
  WORKSPACE_SWITCHED: 'workspace-switched',
  WORKSPACE_SCANNED: 'workspace-scanned',
  FILE_CHANGED: 'file-changed',
  FILE_CREATED: 'file-created',
  FILE_DELETED: 'file-deleted',

  // Note events
  NOTE_CREATED: 'note-created',
  NOTE_UPDATED: 'note-updated',
  NOTE_DELETED: 'note-deleted',
  NOTE_VERSION_RESTORED: 'note-version-restored',

  // Notebook events
  NOTEBOOK_CREATED: 'notebook-created',
  NOTEBOOK_UPDATED: 'notebook-updated',
  NOTEBOOK_DELETED: 'notebook-deleted',

  // Tag events
  TAG_CREATED: 'tag-created',
  TAG_UPDATED: 'tag-updated',
  TAG_DELETED: 'tag-deleted',

  // Attachment events
  ATTACHMENT_ADDED: 'attachment-added',
  ATTACHMENT_DELETED: 'attachment-deleted',

  // Database events
  DB_MIGRATION_PROGRESS: 'db-migration-progress',
  DB_MIGRATION_COMPLETE: 'db-migration-complete',
  DB_BACKUP_PROGRESS: 'db-backup-progress',
  DB_BACKUP_COMPLETE: 'db-backup-complete',
  DB_RESTORE_PROGRESS: 'db-restore-progress',
  DB_RESTORE_COMPLETE: 'db-restore-complete',
  DB_VACUUM_PROGRESS: 'db-vacuum-progress',
  DB_VACUUM_COMPLETE: 'db-vacuum-complete',

  // Settings events
  SETTINGS_CHANGED: 'settings-changed',

  // Topic events
  TOPIC_CREATED: 'topic-created',
  TOPIC_UPDATED: 'topic-updated',
  TOPIC_DELETED: 'topic-deleted',
  NOTE_CLASSIFIED: 'note-classified',
  EMBEDDING_PROGRESS: 'embedding-progress',

  // ML Service status events
  ML_STATUS_CHANGED: 'ml-status-changed',
  ML_OPERATION_STARTED: 'ml-operation-started',
  ML_OPERATION_PROGRESS: 'ml-operation-progress',
  ML_OPERATION_COMPLETED: 'ml-operation-completed',
  ML_OPERATION_ERROR: 'ml-operation-error',
} as const;

// Export all command constants for validation
export const ALL_COMMANDS = [
  ...Object.values(WORKSPACE_COMMANDS),
  ...Object.values(NOTE_COMMANDS),
  ...Object.values(NOTEBOOK_COMMANDS),
  ...Object.values(TAG_COMMANDS),
  ...Object.values(TOPIC_COMMANDS),
  ...Object.values(SEARCH_COMMANDS),
  ...Object.values(ATTACHMENT_COMMANDS),
  ...Object.values(DATABASE_COMMANDS),
  ...Object.values(SETTINGS_COMMANDS),
  ...Object.values(SYSTEM_COMMANDS),
  ...Object.values(GIT_COMMANDS),
  ...Object.values(PERFORMANCE_COMMANDS),
  ...Object.values(QUICK_CAPTURE_COMMANDS),
] as const;

// Export all events as an array
export const ALL_EVENTS = Object.values(EVENTS);

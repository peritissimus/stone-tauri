/**
 * Topic API - IPC channel wrappers for topic/semantic operations
 *
 * Implements: specs/api.ts#TopicAPI
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import { TOPIC_COMMANDS } from '../constants/tauriCommands';
import type {
  Topic,
  TopicWithCount,
  ClassificationResult,
  SimilarNote,
  EmbeddingStatus,
  IpcResponse,
} from '../types';
import { validateResponse } from './validation';
import {
  TopicSchema,
  TopicWithCountSchema,
  ClassificationResultSchema,
  SimilarNoteResultSchema,
  EmbeddingStatusSchema,
  NoteTopicDetailsSchema,
  ClassifyNoteResponseSchema,
  ClassifyAllResponseSchema,
} from './schemas';
import { z } from 'zod';

export const topicAPI = {
  /**
   * Initialize the embedding service
   */
  initialize: async (): Promise<IpcResponse<EmbeddingStatus>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.GET_EMBEDDING_STATUS, {});
    return validateResponse(response, EmbeddingStatusSchema);
  },

  /**
   * Get all topics
   */
  getAll: async (options?: {
    excludeJournal?: boolean;
  }): Promise<IpcResponse<{ topics: TopicWithCount[] }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.GET_ALL, {
      exclude_journal: options?.excludeJournal,
    });
    return validateResponse(response, z.object({ topics: z.array(TopicWithCountSchema) }));
  },

  /**
   * Get a topic by ID
   */
  getById: async (id: string): Promise<IpcResponse<Topic>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.GET_BY_ID, { id });
    return validateResponse(response, TopicSchema);
  },

  /**
   * Create a new topic
   */
  create: async (data: {
    name: string;
    description?: string;
    color?: string;
  }): Promise<IpcResponse<Topic>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.CREATE, { request: data });
    return validateResponse(response, TopicSchema);
  },

  /**
   * Update an existing topic
   */
  update: async (
    id: string,
    data: Partial<{
      name: string;
      description: string;
      color: string;
    }>,
  ): Promise<IpcResponse<Topic>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.UPDATE, {
      request: { id, ...data },
    });
    return validateResponse(response, TopicSchema);
  },

  /**
   * Delete a topic
   */
  delete: async (id: string): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.DELETE, { id });
    return validateResponse(response, z.void());
  },

  /**
   * Get notes for a topic
   */
  getNotesByTopic: async (
    topicId: string,
    options?: { limit?: number; offset?: number; excludeJournal?: boolean },
  ): Promise<IpcResponse<{ notes: unknown[] }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.GET_NOTES_BY_TOPIC, {
      topic_id: topicId,
      ...options,
      exclude_journal: options?.excludeJournal,
    });
    return validateResponse(response, z.object({ notes: z.array(z.unknown()) }));
  },

  /**
   * Get topics for a note
   */
  getTopicsForNote: async (
    noteId: string,
  ): Promise<
    IpcResponse<{
      topics: Array<{
        noteId: string;
        topicId: string;
        confidence: number;
        isManual: boolean;
        createdAt: string;
        topicName: string;
        topicColor: string;
      }>;
    }>
  > => {
    const response = await invokeIpc(TOPIC_COMMANDS.GET_TOPICS_FOR_NOTE, { note_id: noteId });
    return validateResponse(response, z.object({ topics: z.array(NoteTopicDetailsSchema) }));
  },

  /**
   * Assign a topic to a note
   */
  assignToNote: async (noteId: string, topicId: string): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.ASSIGN_TO_NOTE, {
      note_id: noteId,
      topic_id: topicId,
    });
    return validateResponse(response, z.void());
  },

  /**
   * Remove a topic from a note
   */
  removeFromNote: async (noteId: string, topicId: string): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.REMOVE_FROM_NOTE, {
      note_id: noteId,
      topic_id: topicId,
    });
    return validateResponse(response, z.void());
  },

  /**
   * Classify a single note
   */
  classifyNote: async (
    noteId: string,
  ): Promise<IpcResponse<{ noteId: string; topics: ClassificationResult[] }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.CLASSIFY_NOTE, { note_id: noteId });
    return validateResponse(response, ClassifyNoteResponseSchema);
  },

  /**
   * Classify all pending notes
   */
  classifyAll: async (options?: {
    excludeJournal?: boolean;
  }): Promise<IpcResponse<{ processed: number; total: number; failed: number }>> => {
    // Note: classify_all_notes in Rust takes no arguments in the current implementation I read?
    // Let me double check topic_commands.rs
    // "pub async fn classify_all_notes(state: State<'_, AppState>) -> ..."
    // IT TAKES NO ARGUMENTS. So options are ignored?
    // Wait, the interface says `excludeJournal`.
    // If Rust ignores it, passing it doesn't hurt unless strict validation.
    // I'll leave it but just pass empty if needed or pass as is.
    // Actually, passing args to a function that takes none usually results in "invalid args" error in Tauri?
    // No, extra args are usually ignored unless strictly typed?
    // Safest to NOT pass args if Rust signature is empty.
    const response = await invokeIpc(TOPIC_COMMANDS.CLASSIFY_ALL, {});
    return validateResponse(response, ClassifyAllResponseSchema);
  },

  /**
   * Reclassify all notes (force)
   */
  reclassifyAll: async (options?: {
    excludeJournal?: boolean;
  }): Promise<IpcResponse<{ processed: number; total: number; failed: number; skipped: number }>> => {
    // Same here, if Rust command takes no args.
    // Wait, I didn't see reclassify_all_notes in topic_commands.rs.
    // It says "RECLASSIFY_ALL: 'classify_all_notes'" in tauriCommands.ts.
    // So it maps to the same command?
    // If so, it takes no args.
    const response = await invokeIpc(TOPIC_COMMANDS.RECLASSIFY_ALL, {});
    return validateResponse(
      response,
      z.object({
        processed: z.number(),
        total: z.number(),
        failed: z.number(),
        skipped: z.number().optional().default(0), // Rust response might not have skipped if it reuses same struct
      }),
    );
  },

  /**
   * Semantic search
   */
  semanticSearch: async (
    query: string,
    limit?: number,
  ): Promise<IpcResponse<{ results: SimilarNote[] }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.SEMANTIC_SEARCH, { query, limit });
    return validateResponse(response, z.object({ results: z.array(SimilarNoteResultSchema) }));
  },

  /**
   * Find similar notes
   */
  getSimilarNotes: async (
    noteId: string,
    limit?: number,
  ): Promise<IpcResponse<{ similar: SimilarNote[] }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.GET_SIMILAR_NOTES, {
      note_id: noteId,
      limit,
    });
    return validateResponse(response, z.object({ similar: z.array(SimilarNoteResultSchema) }));
  },

  /**
   * Recompute topic centroids
   */
  recomputeCentroids: async (): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.RECOMPUTE_CENTROIDS, {});
    return validateResponse(response, z.void());
  },

  /**
   * Get embedding status
   */
  getEmbeddingStatus: async (): Promise<IpcResponse<EmbeddingStatus>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.GET_EMBEDDING_STATUS, {});
    return validateResponse(response, EmbeddingStatusSchema);
  },
};

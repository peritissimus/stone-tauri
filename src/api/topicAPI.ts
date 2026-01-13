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
  initialize: async (): Promise<IpcResponse<{ success: boolean; ready: boolean }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.INITIALIZE, {});
    return validateResponse(response, z.object({ success: z.boolean(), ready: z.boolean() }));
  },

  /**
   * Get all topics
   */
  getAll: async (options?: {
    excludeJournal?: boolean;
  }): Promise<IpcResponse<{ topics: TopicWithCount[] }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.GET_ALL, options || {});
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
    const response = await invokeIpc(TOPIC_COMMANDS.CREATE, data);
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
    const response = await invokeIpc(TOPIC_COMMANDS.UPDATE, { id, ...data });
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
    const response = await invokeIpc(TOPIC_COMMANDS.GET_NOTES_BY_TOPIC, { topicId, ...options });
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
    const response = await invokeIpc(TOPIC_COMMANDS.GET_TOPICS_FOR_NOTE, { noteId });
    return validateResponse(response, z.object({ topics: z.array(NoteTopicDetailsSchema) }));
  },

  /**
   * Assign a topic to a note
   */
  assignToNote: async (noteId: string, topicId: string): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.ASSIGN_TO_NOTE, { noteId, topicId });
    return validateResponse(response, z.void());
  },

  /**
   * Remove a topic from a note
   */
  removeFromNote: async (noteId: string, topicId: string): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.REMOVE_FROM_NOTE, { noteId, topicId });
    return validateResponse(response, z.void());
  },

  /**
   * Classify a single note
   */
  classifyNote: async (
    noteId: string,
  ): Promise<IpcResponse<{ noteId: string; topics: ClassificationResult[] }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.CLASSIFY_NOTE, { noteId });
    return validateResponse(response, ClassifyNoteResponseSchema);
  },

  /**
   * Classify all pending notes
   */
  classifyAll: async (options?: {
    excludeJournal?: boolean;
  }): Promise<IpcResponse<{ processed: number; total: number; failed: number }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.CLASSIFY_ALL, options || {});
    return validateResponse(response, ClassifyAllResponseSchema);
  },

  /**
   * Reclassify all notes (force)
   */
  reclassifyAll: async (options?: {
    excludeJournal?: boolean;
  }): Promise<IpcResponse<{ processed: number; total: number; failed: number; skipped: number }>> => {
    const response = await invokeIpc(TOPIC_COMMANDS.RECLASSIFY_ALL, options || {});
    return validateResponse(
      response,
      z.object({
        processed: z.number(),
        total: z.number(),
        failed: z.number(),
        skipped: z.number(),
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
    const response = await invokeIpc(TOPIC_COMMANDS.GET_SIMILAR_NOTES, { noteId, limit });
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

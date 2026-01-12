/**
 * Search API - IPC channel wrappers for search operations
 *
 * Implements: specs/api.ts#SearchAPI
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import { SEARCH_COMMANDS } from '../constants/tauriCommands';
import type { SearchResults, IpcResponse } from '../types';

export interface SearchParams {
  query: string;
  limit?: number;
  offset?: number;
}

export interface DateRangeParams {
  startDate: string;
  endDate: string;
  limit?: number;
  offset?: number;
}

export const searchAPI = {
  /**
   * Full-text search
   */
  fullText: (params: SearchParams): Promise<IpcResponse<SearchResults>> =>
    invokeIpc(SEARCH_COMMANDS.FULL_TEXT, params),

  /**
   * Semantic search using embeddings
   */
  semantic: (params: SearchParams): Promise<IpcResponse<SearchResults>> =>
    invokeIpc(SEARCH_COMMANDS.SEMANTIC, params),

  /**
   * Hybrid search (combines full-text and semantic)
   */
  hybrid: (params: SearchParams): Promise<IpcResponse<SearchResults>> =>
    invokeIpc(SEARCH_COMMANDS.HYBRID, params),

  /**
   * Search by tag
   */
  byTag: (
    tagId: string,
    options?: { limit?: number; offset?: number },
  ): Promise<IpcResponse<SearchResults>> =>
    invokeIpc(SEARCH_COMMANDS.BY_TAG, { tagId, ...options }),

  /**
   * Search by date range
   */
  byDateRange: (params: DateRangeParams): Promise<IpcResponse<SearchResults>> =>
    invokeIpc(SEARCH_COMMANDS.BY_DATE_RANGE, params),
};

/**
 * Tag API - IPC channel wrappers for tag operations
 *
 * Implements: specs/api.ts#TagAPI
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import { TAG_COMMANDS } from '../constants/tauriCommands';
import type { Tag, TagWithCount, IpcResponse } from '../types';
import { validateResponse } from './validation';
import { TagSchema, TagWithCountSchema } from './schemas';
import { z } from 'zod';

export interface GetAllTagsParams {
  sort?: 'name' | 'count' | 'recent';
}

export const tagAPI = {
  /**
   * Get all tags
   */
  getAll: async (params?: GetAllTagsParams): Promise<IpcResponse<{ tags: TagWithCount[] }>> => {
    const response = await invokeIpc(TAG_COMMANDS.GET_ALL, params);
    return validateResponse(response, z.object({ tags: z.array(TagWithCountSchema) }));
  },

  /**
   * Create a new tag
   */
  create: async (data: { name: string; color?: string }): Promise<IpcResponse<Tag>> => {
    const response = await invokeIpc(TAG_COMMANDS.CREATE, data);
    return validateResponse(response, TagSchema);
  },

  /**
   * Delete a tag
   */
  delete: async (id: string): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(TAG_COMMANDS.DELETE, { id });
    return validateResponse(response, z.void());
  },

  /**
   * Add tags to a note
   */
  addToNote: async (noteId: string, tagIds: string[]): Promise<IpcResponse<{ tags: TagWithCount[] }>> => {
    const response = await invokeIpc(TAG_COMMANDS.ADD_TO_NOTE, { noteId, tagIds });
    return validateResponse(response, z.object({ tags: z.array(TagWithCountSchema) }));
  },

  /**
   * Remove a tag from a note
   */
  removeFromNote: async (noteId: string, tagId: string): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(TAG_COMMANDS.REMOVE_FROM_NOTE, { noteId, tagId });
    return validateResponse(response, z.void());
  },
};

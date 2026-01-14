/**
 * Notebook API - IPC channel wrappers for notebook operations
 *
 * Implements: specs/api.ts#NotebookAPI
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import { NOTEBOOK_COMMANDS } from '../constants/tauriCommands';
import type { Notebook, IpcResponse } from '../types';
import { validateResponse } from './validation';
import { NotebookSchema, NotebookWithCountSchema } from './schemas';
import { z } from 'zod';

export interface NotebookWithCount extends Notebook {
  note_count: number;
  children?: NotebookWithCount[];
}

export interface GetAllNotebooksParams {
  include_counts?: boolean;
  flat?: boolean;
}

export const notebookAPI = {
  /**
   * Get all notebooks
   */
  getAll: async (
    params?: GetAllNotebooksParams,
  ): Promise<IpcResponse<{ notebooks: NotebookWithCount[] }>> => {
    const response = await invokeIpc(NOTEBOOK_COMMANDS.GET_ALL, params);
    return validateResponse(response, z.object({ notebooks: z.array(NotebookWithCountSchema) }));
  },

  /**
   * Create a new notebook
   */
  create: async (data: {
    name: string;
    parentId?: string;
    icon?: string;
    color?: string;
  }): Promise<IpcResponse<Notebook>> => {
    const response = await invokeIpc(NOTEBOOK_COMMANDS.CREATE, {
      request: {
        name: data.name,
        parentId: data.parentId,
        icon: data.icon,
        color: data.color,
      },
    });
    return validateResponse(response, NotebookSchema);
  },

  /**
   * Update an existing notebook
   */
  update: async (
    id: string,
    data: Partial<{
      name: string;
      icon: string;
      color: string;
    }>,
  ): Promise<IpcResponse<Notebook>> => {
    const response = await invokeIpc(NOTEBOOK_COMMANDS.UPDATE, {
      request: { id, ...data },
    });
    return validateResponse(response, NotebookSchema);
  },

  /**
   * Delete a notebook
   */
  delete: async (id: string, deleteNotes?: boolean): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(NOTEBOOK_COMMANDS.DELETE, {
      request: { id, deleteNotes },
    });
    return validateResponse(response, z.void());
  },

  /**
   * Move a notebook to a new parent or position
   */
  move: async (id: string, parentId?: string, position?: number): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(NOTEBOOK_COMMANDS.MOVE, {
      request: { id, parentId, position },
    });
    return validateResponse(response, z.void());
  },
};

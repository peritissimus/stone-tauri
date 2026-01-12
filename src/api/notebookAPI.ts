/**
 * Notebook API - IPC channel wrappers for notebook operations
 *
 * Implements: specs/api.ts#NotebookAPI
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import { NOTEBOOK_COMMANDS } from '../constants/tauriCommands';
import type { Notebook, IpcResponse } from '../types';

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
  getAll: (
    params?: GetAllNotebooksParams,
  ): Promise<IpcResponse<{ notebooks: NotebookWithCount[] }>> =>
    invokeIpc(NOTEBOOK_COMMANDS.GET_ALL, params),

  /**
   * Create a new notebook
   */
  create: (data: {
    name: string;
    parent_id?: string;
    icon?: string;
    color?: string;
  }): Promise<IpcResponse<Notebook>> => invokeIpc(NOTEBOOK_COMMANDS.CREATE, data),

  /**
   * Update an existing notebook
   */
  update: (
    id: string,
    data: Partial<{
      name: string;
      icon: string;
      color: string;
    }>,
  ): Promise<IpcResponse<Notebook>> => invokeIpc(NOTEBOOK_COMMANDS.UPDATE, { id, ...data }),

  /**
   * Delete a notebook
   */
  delete: (id: string, deleteNotes?: boolean): Promise<IpcResponse<void>> =>
    invokeIpc(NOTEBOOK_COMMANDS.DELETE, { id, delete_notes: deleteNotes }),

  /**
   * Move a notebook to a new parent or position
   */
  move: (id: string, parentId?: string, position?: number): Promise<IpcResponse<void>> =>
    invokeIpc(NOTEBOOK_COMMANDS.MOVE, { id, parent_id: parentId, position }),
};

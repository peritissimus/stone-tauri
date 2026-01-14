/**
 * Workspace API - IPC channel wrappers for workspace operations
 *
 * Implements: specs/api.ts#WorkspaceAPI
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import { WORKSPACE_COMMANDS } from '../constants/tauriCommands';
import type { Workspace, IpcResponse } from '../types';
import { validateResponse } from './validation';
import { WorkspaceSchema } from './schemas';
import { z } from 'zod';

export interface FileTreeNode {
  name: string;
  path: string;
  type: 'file' | 'folder';
  children?: FileTreeNode[];
  noteId?: string;
}

export const workspaceAPI = {
  /**
   * Get all workspaces
   */
  getAll: async (): Promise<IpcResponse<{ workspaces: Workspace[] }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.GET_ALL, {});
    return validateResponse(response, z.object({ workspaces: z.array(WorkspaceSchema) }));
  },

  /**
   * Get the active workspace
   */
  getActive: async (): Promise<IpcResponse<{ workspace?: Workspace }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.GET_ACTIVE, {});
    return validateResponse(response, z.object({ workspace: WorkspaceSchema.optional() }));
  },

  /**
   * Create a new workspace
   */
  create: async (data: { name: string; path: string }): Promise<IpcResponse<Workspace>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.CREATE, {
      request: { name: data.name, folderPath: data.path, setActive: true },
    });
    return validateResponse(response, WorkspaceSchema);
  },

  /**
   * Update a workspace
   */
  update: async (id: string, data: Partial<{ name: string }>): Promise<IpcResponse<Workspace>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.UPDATE, {
      request: { id, ...data },
    });
    return validateResponse(response, WorkspaceSchema);
  },

  /**
   * Create a new workspace
   */
  create: async (data: { name: string; path: string }): Promise<IpcResponse<Workspace>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.CREATE, {
      request: { name: data.name, folderPath: data.path, setActive: true },
    });
    return validateResponse(response, WorkspaceSchema);
  },

  /**
   * Update a workspace
   */
  update: async (id: string, data: Partial<{ name: string }>): Promise<IpcResponse<Workspace>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.UPDATE, {
      request: { id, ...data },
    });
    return validateResponse(response, WorkspaceSchema);
  },

  /**
   * Delete a workspace
   */
  delete: async (id: string): Promise<IpcResponse<void>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.DELETE, { id });
    return validateResponse(response, z.void());
  },

  /**
   * Scan workspace for files
   */
  scan: async (
    workspaceId: string,
  ): Promise<
    IpcResponse<{
      files: Array<{
        path: string;
        relativePath: string;
      }>;
      structure: Array<{
        name: string;
        path: string;
        relativePath: string;
        type: 'file' | 'folder';
        children?: any[];
      }>;
      counts?: Record<string, number>;
    }>
  > => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SCAN, { workspace_id: workspaceId });
    return validateResponse(
      response,
      z.object({
        files: z.array(z.object({
          path: z.string(),
          relativePath: z.string(),
        })),
        structure: z.array(
          z.object({
            name: z.string(),
            path: z.string(),
            relativePath: z.string(),
            type: z.enum(['file', 'folder']),
            children: z.array(z.any()).nullable().optional(),
          }),
        ),
        counts: z.record(z.number()).optional(),
      }),
    );
  },

  /**
   * Sync workspace with filesystem
   */
  sync: async (
    workspaceId?: string,
  ): Promise<
    IpcResponse<{
      workspaceId: string;
      notebooks: { created: number; updated: number; errors: string[] };
      notes: { created: number; updated: number; deleted: number; errors: string[] };
    }>
  > => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SYNC, workspaceId ? { workspace_id: workspaceId } : {});
    return validateResponse(
      response,
      z.object({
        workspaceId: z.string(),
        notebooks: z.object({
          created: z.number(),
          updated: z.number(),
          errors: z.array(z.string()),
        }),
        notes: z.object({
          created: z.number(),
          updated: z.number(),
          deleted: z.number(),
          errors: z.array(z.string()),
        }),
      }),
    );
  },

  /**
   * Create a folder in the workspace
   */
  createFolder: async (name: string, parentPath?: string): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.CREATE_FOLDER, {
      request: { name, parentPath },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Rename a folder
   */
  renameFolder: async (path: string, name: string): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.RENAME_FOLDER, {
      request: { path, name },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Move a folder
   */
  moveFolder: async (
    sourcePath: string,
    destinationPath: string | null,
  ): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.MOVE_FOLDER, {
      request: { sourcePath, destinationPath },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Open folder selection dialog
   */
  selectFolder: async (): Promise<IpcResponse<{ canceled?: boolean; folderPath?: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SELECT_FOLDER, { request: null });
    return validateResponse(
      response,
      z.object({ canceled: z.boolean().optional(), folderPath: z.string().optional() }),
    );
  },

  /**
   * Sync workspace with filesystem
   */
  sync: async (
    workspaceId?: string,
  ): Promise<
    IpcResponse<{
      workspaceId: string;
      notebooks: { created: number; updated: number; errors: string[] };
      notes: { created: number; updated: number; deleted: number; errors: string[] };
    }>
  > => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SYNC, { workspace_id: workspaceId });
    return validateResponse(
      response,
      z.object({
        workspaceId: z.string(),
        notebooks: z.object({
          created: z.number(),
          updated: z.number(),
          errors: z.array(z.string()),
        }),
        notes: z.object({
          created: z.number(),
          updated: z.number(),
          deleted: z.number(),
          errors: z.array(z.string()),
        }),
      }),
    );
  },

  /**
   * Create a folder in the workspace
   */
  createFolder: async (name: string, parentPath?: string): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.CREATE_FOLDER, {
      request: { name, parentPath },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Rename a folder
   */
  renameFolder: async (path: string, name: string): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.RENAME_FOLDER, {
      request: { path, name },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Move a folder
   */
  moveFolder: async (
    sourcePath: string,
    destinationPath: string | null,
  ): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.MOVE_FOLDER, {
      request: { sourcePath, destinationPath },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Open folder selection dialog
   */
  selectFolder: async (): Promise<IpcResponse<{ canceled?: boolean; folderPath?: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SELECT_FOLDER, { request: null });
    return validateResponse(
      response,
      z.object({ canceled: z.boolean().optional(), folderPath: z.string().optional() }),
    );
  },

  /**
   * Create a folder in the workspace
   */
  createFolder: async (name: string, parentPath?: string): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.CREATE_FOLDER, {
      request: { name, parentPath },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Rename a folder
   */
  renameFolder: async (path: string, name: string): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.RENAME_FOLDER, {
      request: { path, name },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Move a folder
   */
  moveFolder: async (
    sourcePath: string,
    destinationPath: string | null,
  ): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.MOVE_FOLDER, {
      request: { sourcePath, destinationPath },
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Open folder selection dialog
   */
  selectFolder: async (): Promise<IpcResponse<{ canceled?: boolean; folderPath?: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SELECT_FOLDER, { request: null });
    return validateResponse(
      response,
      z.object({ canceled: z.boolean().optional(), folderPath: z.string().optional() }),
    );
  },

  /**
   * Rename a folder
   */
  renameFolder: async (path: string, name: string): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.RENAME_FOLDER, {
      path,
      name,
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Delete a folder
   */
  deleteFolder: async (path: string): Promise<IpcResponse<{ success: boolean }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.DELETE_FOLDER, { path });
    return validateResponse(response, z.object({ success: z.boolean() }));
  },

  /**
   * Move a folder
   */
  moveFolder: async (
    sourcePath: string,
    destinationPath: string | null,
  ): Promise<IpcResponse<{ folderPath: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.MOVE_FOLDER, {
      sourcePath,
      destinationPath,
    });
    return validateResponse(response, z.object({ folderPath: z.string() }));
  },

  /**
   * Validate a path
   */
  validatePath: async (path: string): Promise<IpcResponse<{ valid: boolean; message?: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.VALIDATE_PATH, { path });
    return validateResponse(response, z.object({ valid: z.boolean(), message: z.string().optional() }));
  },

  /**
   * Open folder selection dialog
   */
  selectFolder: async (): Promise<IpcResponse<{ canceled?: boolean; folderPath?: string }>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SELECT_FOLDER, {});
    return validateResponse(
      response,
      z.object({ canceled: z.boolean().optional(), folderPath: z.string().optional() }),
    );
  },
};

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
import { logger } from '../utils/logger';

// Permissive scan schema to tolerate backend variations
const ScanResponseSchema = z
  .object({
    files: z
      .array(
        z
          .object({
            path: z.string(),
            relativePath: z.string(),
          })
          .passthrough(),
      )
      .optional(),
    structure: z
      .array(
        z
          .object({
            name: z.string(),
            path: z.string(),
            relativePath: z.string().optional(),
            type: z.enum(['file', 'folder', 'directory', 'dir']),
            children: z.array(z.any()).nullable().optional(),
          })
          .passthrough(),
      )
      .optional(),
    // Use z.unknown() to accept any value type - we sanitize before validation anyway
    counts: z.record(z.string(), z.unknown()).optional(),
    total: z.unknown().optional(),
  })
  .passthrough();

export interface FileTreeNode {
  name: string;
  path: string;
  type: 'file' | 'folder';
  children?: FileTreeNode[] | null;
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
   * Set the active workspace
   */
  setActive: async (id: string): Promise<IpcResponse<Workspace>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SET_ACTIVE, { id });
    return validateResponse(response, WorkspaceSchema);
  },

  /**
   * Create a new workspace
   */
  create: async (data: { name: string; path: string; setActive?: boolean }): Promise<IpcResponse<Workspace>> => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.CREATE, {
      request: { name: data.name, folderPath: data.path, setActive: data.setActive ?? true },
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
        children?: any[] | null;
      }>;
      counts: Record<string, number>;
      total: number;
    }>
  > => {
    try {
      // Backend expects camelCase `workspaceId` (see error "missing required key workspaceId")
      const params = {
        workspaceId,
        // Send both casings for compatibility with backend expectations
        workspace_id: workspaceId,
      };
      logger.info('[workspaceAPI.scan] invoking', params);
      const response = await invokeIpc(WORKSPACE_COMMANDS.SCAN, params);
      logger.info('[workspaceAPI.scan] raw response', response);

      // Sanitize counts/total: replace NaN/invalid with 0 to satisfy downstream usage
      if (response && typeof response === 'object' && (response as any).data) {
        const data: any = (response as any).data;
        if (data.counts && typeof data.counts === 'object') {
          const sanitized: Record<string, number> = {};
          Object.entries(data.counts).forEach(([key, value]) => {
            const num = Number(value);
            sanitized[key] = Number.isFinite(num) ? num : 0;
          });
          data.counts = sanitized;
        }
        if (data.total !== undefined) {
          const totalNum = Number(data.total);
          data.total = Number.isFinite(totalNum) ? totalNum : 0;
        }
      }

      // Validate with permissive schema (z.any() for counts avoids blocking on NaN/strings)
      return validateResponse(response, ScanResponseSchema);
    } catch (error) {
      logger.error('[workspaceAPI.scan] failed', { error, workspaceId });
      return {
        success: false,
        error: {
          code: 'CLIENT_VALIDATION',
          message: error instanceof Error ? error.message : 'Failed to scan workspace',
        },
      } as IpcResponse<any>;
    }
  },

  /**
   * Sync workspace with filesystem
   */
  sync: async (
    workspaceId?: string,
  ): Promise<
    IpcResponse<{
      notes: { created: number; updated: number; deleted: number };
      durationMs: number;
    }>
  > => {
    const response = await invokeIpc(WORKSPACE_COMMANDS.SYNC, workspaceId ? { workspace_id: workspaceId } : {});
    return validateResponse(
      response,
      z.object({
        notes: z.object({
          created: z.number(),
          updated: z.number(),
          deleted: z.number(),
        }),
        durationMs: z.number(),
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
      request: { sourcePath, destinationPath },
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
    const response = await invokeIpc(WORKSPACE_COMMANDS.SELECT_FOLDER, { request: null });
    return validateResponse(
      response,
      z.object({ canceled: z.boolean().optional(), folderPath: z.string().optional() }),
    );
  },
};


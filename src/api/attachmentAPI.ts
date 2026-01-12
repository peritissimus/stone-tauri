/**
 * Attachment API - IPC channel wrappers for attachment operations
 *
 * Implements: specs/api.ts#AttachmentAPI
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import { ATTACHMENT_COMMANDS } from '../constants/tauriCommands';
import type { Attachment, IpcResponse } from '../types';

export const attachmentAPI = {
  /**
   * Get all attachments for a note
   */
  getAll: (noteId: string): Promise<IpcResponse<{ attachments: Attachment[] }>> =>
    invokeIpc(ATTACHMENT_COMMANDS.GET_ALL, { noteId }),

  /**
   * Add an attachment to a note
   */
  add: (noteId: string, filePath: string): Promise<IpcResponse<Attachment>> =>
    invokeIpc(ATTACHMENT_COMMANDS.ADD, { noteId, filePath }),

  /**
   * Delete an attachment
   */
  delete: (id: string): Promise<IpcResponse<void>> => invokeIpc(ATTACHMENT_COMMANDS.DELETE, { id }),

  /**
   * Upload an image and attach to note
   */
  uploadImage: (
    noteId: string,
    imageData: string,
    filename: string,
  ): Promise<IpcResponse<{ url: string; attachment: Attachment }>> =>
    invokeIpc(ATTACHMENT_COMMANDS.UPLOAD_IMAGE, {
      noteId,
      imageData,
      filename,
    }),
};

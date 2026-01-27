/**
 * Quick Capture API - IPC channel wrappers for quick capture operations
 *
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from "../lib/tauri-ipc";
import { QUICK_CAPTURE_COMMANDS } from "../constants/tauriCommands";
import type { IpcResponse } from "../types";

export interface AppendToJournalResponse {
  noteId: string;
  appended: boolean;
}

export interface PanelOperationResult {
  success: boolean;
  state: string;
  error: string | null;
}

export const quickCaptureAPI = {
  /**
   * Append text to today's journal entry
   */
  appendToJournal: (
    text: string,
    workspaceId?: string,
  ): Promise<IpcResponse<AppendToJournalResponse>> =>
    invokeIpc(QUICK_CAPTURE_COMMANDS.APPEND_TO_JOURNAL, {
      content: text,
      workspace_id: workspaceId,
    }),

  /**
   * Hide the quick capture window
   */
  hide: (): Promise<IpcResponse<PanelOperationResult>> =>
    invokeIpc(QUICK_CAPTURE_COMMANDS.HIDE, {}),

  /**
   * Get the current state of the quick capture panel (for debugging)
   */
  getState: (): Promise<IpcResponse<string>> =>
    invokeIpc(QUICK_CAPTURE_COMMANDS.GET_STATE, {}),
};

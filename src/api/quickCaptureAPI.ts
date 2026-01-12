/**
 * Quick Capture API - IPC channel wrappers for quick capture operations
 *
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import { QUICK_CAPTURE_COMMANDS } from '../constants/tauriCommands';
import type { IpcResponse, Note } from '../types';

export const quickCaptureAPI = {
  /**
   * Append text to today's journal entry
   */
  appendToJournal: (text: string): Promise<IpcResponse<{ note: Note }>> =>
    invokeIpc(QUICK_CAPTURE_COMMANDS.APPEND_TO_JOURNAL, { text }),
};

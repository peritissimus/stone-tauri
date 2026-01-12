/**
 * Tauri IPC Utilities - Centralized IPC invocation and response handling
 *
 * This module provides type-safe wrappers for Tauri IPC communication,
 * replacing the Electron IPC implementation with Tauri's invoke API.
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '../utils/logger';

/**
 * IPC Response type matching the backend format
 */
export interface IpcResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: unknown;
  };
}

/**
 * Result of handling an IPC response
 */
export type IpcResult<T> = { success: true; data: T } | { success: false; error: string };

/**
 * Options for IPC invocation
 */
export interface InvokeOptions {
  /** Log prefix for debugging */
  logPrefix?: string;
  /** Whether to log the request/response */
  debug?: boolean;
}

/**
 * Handle an IPC response with consistent error extraction
 *
 * @param response - The IPC response from Tauri invoke
 * @param fallbackError - Error message to use if none is provided
 * @returns Normalized result with success flag and data or error
 *
 * @example
 * const response = await invokeIpc<Note>('get_note', { id });
 * const result = handleIpcResponse(response, 'Failed to load note');
 * if (result.success) {
 *   // result.data is Note
 * } else {
 *   // result.error is string
 * }
 */
export function handleIpcResponse<T>(
  response: IpcResponse<T>,
  fallbackError: string,
): IpcResult<T> {
  if (response.success && response.data !== undefined) {
    return { success: true, data: response.data };
  }
  return {
    success: false,
    error: response.error?.message || fallbackError,
  };
}

/**
 * Handle an IPC response that only returns success/failure (no data)
 *
 * @param response - The IPC response from Tauri invoke
 * @param fallbackError - Error message to use if none is provided
 * @returns Normalized result with success flag and optional error
 */
export function handleIpcVoidResponse(
  response: IpcResponse<unknown>,
  fallbackError: string,
): { success: boolean; error?: string } {
  if (response.success) {
    return { success: true };
  }
  return {
    success: false,
    error: response.error?.message || fallbackError,
  };
}

/**
 * Type-safe IPC invocation wrapper with optional logging
 *
 * This function wraps Tauri's invoke API to match the Electron IPC interface,
 * including automatic error handling and response wrapping.
 *
 * @param command - The Tauri command to invoke
 * @param params - Parameters to pass to the handler
 * @param options - Optional configuration for logging
 * @returns Promise resolving to the IPC response
 *
 * @example
 * const response = await invokeIpc<Note>('get_note', { id }, {
 *   logPrefix: '[NoteAPI.loadNote]',
 *   debug: true
 * });
 */
export async function invokeIpc<T>(
  command: string,
  params?: unknown,
  options?: InvokeOptions,
): Promise<IpcResponse<T>> {
  const { logPrefix, debug } = options || {};

  if (debug && logPrefix) {
    logger.info(`${logPrefix} invoking ${command}`, params);
  }

  try {
    // Tauri's invoke returns the data directly or throws on error
    // We need to wrap it in our IpcResponse format for consistency
    const data = await invoke<T>(command, params || {});

    if (debug && logPrefix) {
      logger.info(`${logPrefix} response`, { success: true });
    }

    return {
      success: true,
      data,
    };
  } catch (error) {
    // Tauri throws errors as strings or Error objects
    const errorMessage = error instanceof Error ? error.message : String(error);

    if (debug && logPrefix) {
      logger.error(`${logPrefix} error`, error);
    }

    return {
      success: false,
      error: {
        code: 'TAURI_ERROR',
        message: errorMessage,
        details: error,
      },
    };
  }
}

/**
 * Invoke IPC and handle response in one call
 *
 * @param command - The Tauri command to invoke
 * @param params - Parameters to pass to the handler
 * @param fallbackError - Error message if the call fails
 * @param options - Optional configuration for logging
 * @returns Normalized result with success flag and data or error
 *
 * @example
 * const result = await invokeAndHandle<Note>(
 *   'get_note',
 *   { id },
 *   'Failed to load note'
 * );
 * if (result.success) {
 *   updateNote(result.data);
 * } else {
 *   setError(result.error);
 * }
 */
export async function invokeAndHandle<T>(
  command: string,
  params: unknown,
  fallbackError: string,
  options?: InvokeOptions,
): Promise<IpcResult<T>> {
  try {
    const response = await invokeIpc<T>(command, params, options);
    return handleIpcResponse(response, fallbackError);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : fallbackError;
    if (options?.logPrefix) {
      logger.error(`${options.logPrefix} error`, error);
    }
    return { success: false, error: errorMessage };
  }
}

/**
 * Invoke IPC for void operations (delete, toggle, etc.)
 *
 * @param command - The Tauri command to invoke
 * @param params - Parameters to pass to the handler
 * @param fallbackError - Error message if the call fails
 * @param options - Optional configuration for logging
 * @returns Object with success flag and optional error
 */
export async function invokeAndHandleVoid(
  command: string,
  params: unknown,
  fallbackError: string,
  options?: InvokeOptions,
): Promise<{ success: boolean; error?: string }> {
  try {
    const response = await invokeIpc<unknown>(command, params, options);
    return handleIpcVoidResponse(response, fallbackError);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : fallbackError;
    if (options?.logPrefix) {
      logger.error(`${options.logPrefix} error`, error);
    }
    return { success: false, error: errorMessage };
  }
}

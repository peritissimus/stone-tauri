/**
 * Event Subscription Utilities - Centralized event listener management
 *
 * This module provides type-safe wrappers for Tauri event subscriptions.
 */

import { listen, UnlistenFn } from '@tauri-apps/api/event';

/**
 * Event handler function type
 */
export type EventHandler<T = unknown> = (payload: T) => void;

/**
 * Unsubscribe function returned when subscribing to an event
 */
export type Unsubscribe = () => void;

/**
 * Subscribe to a Tauri event from the backend
 *
 * @param event - The event name
 * @param handler - Callback function to handle the event
 * @returns Promise that resolves to an unsubscribe function
 *
 * @example
 * useEffect(() => {
 *   let unsubscribe: Unsubscribe;
 *   subscribe('note-updated', (payload) => {
 *     console.log('Note updated:', payload);
 *   }).then(unsub => { unsubscribe = unsub; });
 *   return () => unsubscribe?.();
 * }, []);
 */
export async function subscribe<T = unknown>(
  event: string,
  handler: EventHandler<T>
): Promise<Unsubscribe> {
  const unlisten = await listen<T>(event, (event) => {
    handler(event.payload);
  });
  return unlisten;
}

/**
 * Subscribe to multiple events with a single handler
 *
 * @param eventNames - Array of event names
 * @param handler - Callback function to handle all events
 * @returns Promise that resolves to an unsubscribe function
 *
 * @example
 * useEffect(() => {
 *   let unsubscribe: Unsubscribe;
 *   subscribeMany(
 *     ['file-created', 'file-changed', 'file-deleted'],
 *     () => refreshFileTree()
 *   ).then(unsub => { unsubscribe = unsub; });
 *   return () => unsubscribe?.();
 * }, []);
 */
export async function subscribeMany<T = unknown>(
  eventNames: string[],
  handler: EventHandler<T>
): Promise<Unsubscribe> {
  const unsubscribers = await Promise.all(
    eventNames.map((event) => subscribe<T>(event, handler))
  );
  return () => unsubscribers.forEach((unsub) => unsub());
}

/**
 * Pre-configured event subscribers for common events
 *
 * Note: These event names are converted from Electron's colon format (e.g., 'notes:updated')
 * to Tauri's kebab-case format (e.g., 'note-updated')
 */
export const events = {
  /**
   * Subscribe to file system events
   */
  onFileCreated: <T = unknown>(handler: EventHandler<T>) => subscribe<T>('file-created', handler),

  onFileChanged: <T = unknown>(handler: EventHandler<T>) => subscribe<T>('file-changed', handler),

  onFileDeleted: <T = unknown>(handler: EventHandler<T>) => subscribe<T>('file-deleted', handler),

  /**
   * Subscribe to note events
   */
  onNoteUpdated: <T = unknown>(handler: EventHandler<T>) => subscribe<T>('note-updated', handler),

  onNoteDeleted: <T = unknown>(handler: EventHandler<T>) => subscribe<T>('note-deleted', handler),

  /**
   * Subscribe to workspace events
   */
  onWorkspaceUpdated: <T = unknown>(handler: EventHandler<T>) =>
    subscribe<T>('workspace-updated', handler),

  /**
   * Subscribe to all file system events with a single handler
   */
  onFileSystemChange: <T = unknown>(handler: EventHandler<T>) =>
    subscribeMany<T>(['file-created', 'file-changed', 'file-deleted'], handler),
};

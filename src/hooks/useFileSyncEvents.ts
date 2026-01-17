/**
 * File Sync Events Hook - Subscribe to file watcher events
 *
 * Listens for file system changes detected by the file watcher
 * and triggers appropriate UI updates
 */

import { useEffect, useRef } from 'react';
import { subscribe } from '@/lib/events';
import { logger } from '@/utils/logger';

export type FileSyncOperation = 'created' | 'updated' | 'deleted';

export interface FileSyncEvent {
  timestamp: string;
  file_path: string;
  operation: FileSyncOperation;
}

export interface FileSyncEventHandlers {
  onFileUpdated?: (event: FileSyncEvent) => void;
  onFileCreated?: (event: FileSyncEvent) => void;
  onFileDeleted?: (event: FileSyncEvent) => void;
}

/**
 * Subscribe to file sync events from the file watcher
 *
 * @param handlers - Event handler callbacks
 *
 * @example
 * useFileSyncEvents({
 *   onFileUpdated: (event) => {
 *     if (isCurrentlyOpenFile(event.file_path)) {
 *       reloadFileContent();
 *     }
 *   }
 * });
 */
export function useFileSyncEvents(handlers: FileSyncEventHandlers): void {
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  useEffect(() => {
    const unsubscribe = subscribe('file:synced', (payload: unknown) => {
      const event = payload as FileSyncEvent;

      logger.info('[useFileSyncEvents] File sync event received', {
        file_path: event.file_path,
        operation: event.operation,
      });

      // Dispatch to appropriate handler
      switch (event.operation) {
        case 'created':
          handlersRef.current.onFileCreated?.(event);
          break;
        case 'updated':
          handlersRef.current.onFileUpdated?.(event);
          break;
        case 'deleted':
          handlersRef.current.onFileDeleted?.(event);
          break;
      }
    });

    return () => {
      unsubscribe.then((unsub) => unsub());
    };
  }, []);
}

/**
 * File Events Hook - Subscribe to file system events from main process
 */

import { useEffect, useRef } from 'react';
import { events } from '@/lib/events';

export interface FileEventHandlers {
  onCreated?: (payload: unknown) => void;
  onChanged?: (payload: unknown) => void;
  onDeleted?: (payload: unknown) => void;
}

/**
 * Subscribe to file system events
 *
 * @param handlers - Event handler callbacks
 *
 * @example
 * useFileEvents({
 *   onCreated: () => refreshList(),
 *   onChanged: () => refreshList(),
 *   onDeleted: () => refreshList(),
 * });
 */
export function useFileEvents(handlers: FileEventHandlers): void {
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  useEffect(() => {
    let unsubscribers: (() => void)[] = [];

    const setupSubscriptions = async () => {
      if (handlersRef.current.onCreated) {
        const unsub = await events.onFileCreated((payload) => handlersRef.current.onCreated?.(payload));
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onChanged) {
        const unsub = await events.onFileChanged((payload) => handlersRef.current.onChanged?.(payload));
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onDeleted) {
        const unsub = await events.onFileDeleted((payload) => handlersRef.current.onDeleted?.(payload));
        unsubscribers.push(unsub);
      }
    };

    setupSubscriptions();

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }, []);
}

/**
 * Subscribe to all file system events with a single handler
 *
 * @param handler - Handler called for any file system event
 *
 * @example
 * useFileEventsAll(() => refreshFileTree());
 */
export function useFileEventsAll(handler: (payload: unknown) => void): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let unsub: (() => void) | undefined;

    const setupSubscription = async () => {
      unsub = await events.onFileSystemChange((payload) => handlerRef.current(payload));
    };

    setupSubscription();

    return () => {
      unsub?.();
    };
  }, []);
}

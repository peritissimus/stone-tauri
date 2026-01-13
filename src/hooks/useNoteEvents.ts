/**
 * Note Events Hook - Subscribe to note events from main process
 */

import { useEffect, useRef } from 'react';
import { subscribe } from '@/lib/events';
import { EVENTS } from '@/constants/tauriCommands';

export interface NoteEventHandlers {
  onCreated?: (payload: unknown) => void;
  onUpdated?: (payload: unknown) => void;
  onDeleted?: (payload: unknown) => void;
  onVersionRestored?: (payload: unknown) => void;
}

/**
 * Subscribe to note events
 *
 * @param handlers - Event handler callbacks
 *
 * @example
 * useNoteEvents({
 *   onUpdated: (payload) => handleNoteUpdate(payload),
 *   onDeleted: (payload) => handleNoteDelete(payload),
 * });
 */
export function useNoteEvents(handlers: NoteEventHandlers): void {
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  useEffect(() => {
    let unsubscribers: (() => void)[] = [];

    const setupSubscriptions = async () => {
      if (handlersRef.current.onCreated) {
        const unsub = await subscribe(EVENTS.NOTE_CREATED, (payload) => handlersRef.current.onCreated?.(payload));
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onUpdated) {
        const unsub = await subscribe(EVENTS.NOTE_UPDATED, (payload) => handlersRef.current.onUpdated?.(payload));
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onDeleted) {
        const unsub = await subscribe(EVENTS.NOTE_DELETED, (payload) => handlersRef.current.onDeleted?.(payload));
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onVersionRestored) {
        const unsub = await subscribe(EVENTS.NOTE_VERSION_RESTORED, (payload) =>
          handlersRef.current.onVersionRestored?.(payload),
        );
        unsubscribers.push(unsub);
      }
    };

    setupSubscriptions();

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }, []);
}

/**
 * Workspace Events Hook - Subscribe to workspace events from main process
 */

import { useEffect, useRef } from 'react';
import { subscribe } from '@/lib/events';
import { EVENTS } from '@/constants/tauriCommands';

export interface WorkspaceEventHandlers {
  onCreated?: (payload: unknown) => void;
  onUpdated?: (payload: unknown) => void;
  onDeleted?: (payload: unknown) => void;
  onSwitched?: (payload: unknown) => void;
  onScanned?: (payload: unknown) => void;
}

/**
 * Subscribe to workspace events
 *
 * @param handlers - Event handler callbacks
 *
 * @example
 * useWorkspaceEvents({
 *   onUpdated: () => refreshWorkspace();
 *   onSwitched: (payload) => handleWorkspaceSwitch(payload);
 * });
 */
export function useWorkspaceEvents(handlers: WorkspaceEventHandlers): void {
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  useEffect(() => {
    let unsubscribers: (() => void)[] = [];

    const setupSubscriptions = async () => {
      if (handlersRef.current.onCreated) {
        const unsub = await subscribe(EVENTS.WORKSPACE_CREATED, (payload) => handlersRef.current.onCreated?.(payload));
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onUpdated) {
        const unsub = await subscribe(EVENTS.WORKSPACE_UPDATED, (payload) => handlersRef.current.onUpdated?.(payload));
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onDeleted) {
        const unsub = await subscribe(EVENTS.WORKSPACE_DELETED, (payload) => handlersRef.current.onDeleted?.(payload));
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onSwitched) {
        const unsub = await subscribe(EVENTS.WORKSPACE_SWITCHED, (payload) =>
          handlersRef.current.onSwitched?.(payload),
        );
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onScanned) {
        const unsub = await subscribe(EVENTS.WORKSPACE_SCANNED, (payload) => handlersRef.current.onScanned?.(payload));
        unsubscribers.push(unsub);
      }
    };

    setupSubscriptions();

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }, []);
}

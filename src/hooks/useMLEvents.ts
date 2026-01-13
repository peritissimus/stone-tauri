/**
 * ML Events Hook - Subscribe to ML service events from main process
 */

import { useEffect, useRef } from 'react';
import { subscribe } from '@/lib/events';
import { EVENTS } from '@/constants/tauriCommands';
import type {
  MLStatusChangedPayload,
  MLOperationStartedPayload,
  MLOperationProgressPayload,
  MLOperationCompletedPayload,
  MLOperationErrorPayload,
} from '@/types/mlStatus';

export interface MLEventHandlers {
  onStatusChanged?: (payload: MLStatusChangedPayload) => void;
  onOperationStarted?: (payload: MLOperationStartedPayload) => void;
  onOperationProgress?: (payload: MLOperationProgressPayload) => void;
  onOperationCompleted?: (payload: MLOperationCompletedPayload) => void;
  onOperationError?: (payload: MLOperationErrorPayload) => void;
}

/**
 * Subscribe to ML service events
 *
 * @param handlers - Event handler callbacks
 *
 * @example
 * useMLEvents({
 *   onStatusChanged: (payload) => updateStatus(payload),
 *   onOperationProgress: (payload) => updateProgress(payload),
 * });
 */
export function useMLEvents(handlers: MLEventHandlers): void {
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  useEffect(() => {
    let unsubscribers: (() => void)[] = [];

    const setupSubscriptions = async () => {
      if (handlersRef.current.onStatusChanged) {
        const unsub = await subscribe(EVENTS.ML_STATUS_CHANGED, (payload) =>
          handlersRef.current.onStatusChanged?.(payload as MLStatusChangedPayload),
        );
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onOperationStarted) {
        const unsub = await subscribe(EVENTS.ML_OPERATION_STARTED, (payload) =>
          handlersRef.current.onOperationStarted?.(payload as MLOperationStartedPayload),
        );
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onOperationProgress) {
        const unsub = await subscribe(EVENTS.ML_OPERATION_PROGRESS, (payload) =>
          handlersRef.current.onOperationProgress?.(payload as MLOperationProgressPayload),
        );
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onOperationCompleted) {
        const unsub = await subscribe(EVENTS.ML_OPERATION_COMPLETED, (payload) =>
          handlersRef.current.onOperationCompleted?.(payload as MLOperationCompletedPayload),
        );
        unsubscribers.push(unsub);
      }

      if (handlersRef.current.onOperationError) {
        const unsub = await subscribe(EVENTS.ML_OPERATION_ERROR, (payload) =>
          handlersRef.current.onOperationError?.(payload as MLOperationErrorPayload),
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

/**
 * Subscribe to all ML events and update the ML status store
 * This is a convenience hook that wires up all ML events to the store
 *
 * @example
 * // In App.tsx or a top-level component
 * useMLEventsSync();
 */
export function useMLEventsSync(): void {
  // Import store lazily to avoid circular dependencies
  useEffect(() => {
    let unsubscribers: (() => void)[] = [];

    // Dynamically import to get store actions
    const setupSubscriptions = async () => {
      const { useMLStatusStore } = await import('@/stores/mlStatusStore');
      const store = useMLStatusStore.getState();

      const unsub1 = await subscribe(EVENTS.ML_STATUS_CHANGED, (payload) => {
        store.setServiceStatus(payload as MLStatusChangedPayload);
      });
      unsubscribers.push(unsub1);

      const unsub2 = await subscribe(EVENTS.ML_OPERATION_STARTED, (payload) => {
        store.startOperation(payload as MLOperationStartedPayload);
      });
      unsubscribers.push(unsub2);

      const unsub3 = await subscribe(EVENTS.ML_OPERATION_PROGRESS, (payload) => {
        store.updateProgress(payload as MLOperationProgressPayload);
      });
      unsubscribers.push(unsub3);

      const unsub4 = await subscribe(EVENTS.ML_OPERATION_COMPLETED, (payload) => {
        store.completeOperation(payload as MLOperationCompletedPayload);
      });
      unsubscribers.push(unsub4);

      const unsub5 = await subscribe(EVENTS.ML_OPERATION_ERROR, (payload) => {
        store.failOperation(payload as MLOperationErrorPayload);
      });
      unsubscribers.push(unsub5);
    };

    setupSubscriptions();

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }, []);
}

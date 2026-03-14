/**
 * Autosave Hook - Provides debounced save functionality
 *
 * Clean Architecture: Application layer for save orchestration
 * Separates debouncing logic from UI components
 */

import { useCallback, useEffect, useRef } from 'react';
import { logger } from '@/utils/logger';

interface UseAutosaveOptions<T> {
  /**
   * The function to call when saving (e.g., API call)
   */
  saveFn: (data: T) => Promise<void>;

  /**
   * Delay in milliseconds before saving
   * @default 500
   */
  delay?: number;

  /**
   * Optional callback when save fails
   */
  onError?: (error: unknown) => void;
}

/**
 * Hook that provides debounced autosave functionality
 * Useful for title changes, content updates, etc.
 *
 * @example
 * const { saveDebounced } = useAutosave({
 *   saveFn: async (data) => await updateNote(noteId, data),
 *   delay: 500,
 * });
 *
 * // In your component
 * const handleTitleChange = (title: string) => {
 *   setTitle(title);
 *   saveDebounced({ title });
 * };
 */
export function useAutosave<T = Record<string, any>>({
  saveFn,
  delay = 500,
  onError,
}: UseAutosaveOptions<T>) {
  const timeoutRef = useRef<number | null>(null);
  const isSavingRef = useRef(false);

  /**
   * Saves data after a delay, cancelling any pending saves
   */
  const saveDebounced = useCallback(
    (data: T) => {
      // Clear existing timeout
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      // Set new timeout
      timeoutRef.current = window.setTimeout(async () => {
        timeoutRef.current = null;
        isSavingRef.current = true;
        try {
          await saveFn(data);
        } catch (error) {
          logger.error('Autosave failed:', error);
          onError?.(error);
        } finally {
          isSavingRef.current = false;
        }
      }, delay);
    },
    [saveFn, delay, onError],
  );

  /**
   * Immediately saves without debouncing
   */
  const saveImmediate = useCallback(
    async (data: T) => {
      // Clear pending debounced save
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }

      isSavingRef.current = true;
      try {
        await saveFn(data);
      } catch (error) {
        logger.error('Immediate save failed:', error);
        onError?.(error);
      } finally {
        isSavingRef.current = false;
      }
    },
    [saveFn, onError],
  );

  /**
   * Cancels any pending save
   */
  const cancel = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  /**
   * Cleanup on unmount
   */
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return {
    saveDebounced,
    saveImmediate,
    cancel,
    isPending: () => timeoutRef.current !== null,
    isSaving: () => isSavingRef.current,
  };
}

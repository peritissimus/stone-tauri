/**
 * Quick Capture API Hook - React hook for quick capture operations
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { quickCaptureAPI } from '@/api';
import { logger } from '@/utils/logger';

export function useQuickCaptureAPI() {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const isMountedRef = useRef(true);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const appendToJournal = useCallback(
    async (text: string, workspaceId?: string) => {
      if (!text.trim() || isSubmitting) return null;

      if (isMountedRef.current) {
        setIsSubmitting(true);
        setError(null);
      }

      try {
        const response = await quickCaptureAPI.appendToJournal(text.trim(), workspaceId);
        if (response.success && response.data) {
          logger.info('[useQuickCaptureAPI] Appended to journal', {
            noteId: response.data.noteId,
            appended: response.data.appended,
          });
          return response.data;
        } else {
          const errorMessage = response.error?.message || 'Failed to append to journal';
          logger.error('[useQuickCaptureAPI] Failed:', errorMessage);
          if (isMountedRef.current) setError(errorMessage);
          throw new Error(errorMessage);
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to append to journal';
        logger.error('[useQuickCaptureAPI] Error:', err);
        if (isMountedRef.current) setError(errorMessage);
        throw err instanceof Error ? err : new Error(errorMessage);
      } finally {
        if (isMountedRef.current) setIsSubmitting(false);
      }
    },
    [isSubmitting],
  );

  const hide = useCallback(async () => {
    try {
      const response = await quickCaptureAPI.hide();
      if (!response.success) {
        logger.warn('[useQuickCaptureAPI] Failed to hide window:', response.error?.message);
      }
      return response;
    } catch (err) {
      logger.error('[useQuickCaptureAPI] Error hiding window:', err);
      throw err;
    }
  }, []);

  const getState = useCallback(async () => {
    try {
      const response = await quickCaptureAPI.getState();
      return response;
    } catch (err) {
      logger.error('[useQuickCaptureAPI] Error getting state:', err);
      throw err;
    }
  }, []);

  return {
    appendToJournal,
    hide,
    getState,
    isSubmitting,
    error,
  };
}

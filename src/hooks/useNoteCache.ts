/**
 * Note Cache Service Hook - Provides cached access to notes for autocomplete
 *
 * Clean Architecture: Service layer for caching infrastructure
 * Separates cache management from business logic and UI
 */

import { useCallback, useRef } from 'react';
import { noteAPI } from '@/api';
import { Note } from '@/types';
import { logger } from '@/utils/logger';

interface NoteCacheEntry {
  notes: Note[];
  timestamp: number;
}

const NOTES_CACHE_TTL_MS = 30000; // 30 seconds

/**
 * Hook that manages a cache of notes for autocomplete functionality
 * Provides methods to fetch, filter, and invalidate the cache
 */
export function useNoteCache() {
  const cacheRef = useRef<NoteCacheEntry | null>(null);

  /**
   * Invalidates the cache, forcing a fresh fetch on next access
   */
  const invalidate = useCallback(() => {
    cacheRef.current = null;
  }, []);

  /**
   * Fetches all non-archived notes, using cache if valid
   */
  const fetchNotes = useCallback(async (): Promise<Note[]> => {
    try {
      const now = Date.now();

      // Use cache if valid
      if (!cacheRef.current || now - cacheRef.current.timestamp > NOTES_CACHE_TTL_MS) {
        const response = await noteAPI.getAll({ includeArchived: false });

        if (response.success && response.data) {
          cacheRef.current = {
            notes: response.data.notes || [],
            timestamp: now,
          };
        } else {
          logger.warn('Failed to fetch notes for cache');
          return [];
        }
      }

      return cacheRef.current.notes;
    } catch (error) {
      logger.error('Failed to fetch notes for cache:', error);
      return [];
    }
  }, []);

  /**
   * Fetches and filters notes by query, with sorting by relevance
   * Returns formatted suggestions for autocomplete
   */
  const fetchNotesForAutocomplete = useCallback(
    async (query: string) => {
      try {
        const notes = await fetchNotes();
        const lowerQuery = query.toLowerCase();

        // Filter notes by query
        const filtered = query
          ? notes.filter((note) => note.title?.toLowerCase().includes(lowerQuery))
          : notes;

        // Sort by relevance (title starts with query first, then contains)
        const sorted = filtered.sort((a, b) => {
          const aTitle = (a.title || '').toLowerCase();
          const bTitle = (b.title || '').toLowerCase();
          const aStarts = aTitle.startsWith(lowerQuery);
          const bStarts = bTitle.startsWith(lowerQuery);

          if (aStarts && !bStarts) return -1;
          if (!aStarts && bStarts) return 1;
          return aTitle.localeCompare(bTitle);
        });

        // Limit results
        return sorted.slice(0, 10).map((note) => ({
          id: note.id,
          title: note.title || 'Untitled',
          filePath: note.filePath,
          note,
        }));
      } catch (error) {
        logger.error('Failed to fetch notes for autocomplete:', error);
        return [];
      }
    },
    [fetchNotes],
  );

  return {
    fetchNotes,
    fetchNotesForAutocomplete,
    invalidate,
  };
}

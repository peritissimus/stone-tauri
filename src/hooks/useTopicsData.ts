/**
 * useTopicsData Hook - Manages topic loading, search, and filtering
 */

import { useEffect, useState, useCallback, useMemo } from 'react';
import { useTopicStore } from '@/stores/topicStore';
import { useTopicAPI } from '@/hooks/useTopicAPI';

interface TopicNote {
  id: string;
  title: string;
  confidence?: number;
  isManual?: boolean;
}

export function useTopicsData() {
  const {
    topics,
    selectedTopicId,
    embeddingStatus,
    searchResults,
    searchQuery,
    classifying,
    error,
    selectTopic,
    setSearchQuery,
  } = useTopicStore();

  const {
    initialize,
    loadTopics,
    createTopic,
    semanticSearch,
    reclassifyAllNotes,
    getEmbeddingStatus,
    getNotesForTopic,
  } = useTopicAPI();

  const [searchInput, setSearchInput] = useState('');
  const [initializing, setInitializing] = useState(true);
  const [topicNotes, setTopicNotes] = useState<TopicNote[]>([]);
  const [loadingNotes, setLoadingNotes] = useState(false);
  const [excludeJournal, setExcludeJournal] = useState(true);

  // Initialize topics
  useEffect(() => {
    const init = async () => {
      setInitializing(true);
      await initialize();
      await loadTopics({ excludeJournal });
      await getEmbeddingStatus();
      setInitializing(false);
    };
    init();
  }, [initialize, loadTopics, getEmbeddingStatus, excludeJournal]);

  // Load notes for selected topic
  useEffect(() => {
    if (selectedTopicId) {
      setLoadingNotes(true);
      getNotesForTopic(selectedTopicId, { excludeJournal })
        .then((notes) => setTopicNotes(notes as TopicNote[]))
        .finally(() => setLoadingNotes(false));
    } else {
      setTopicNotes([]);
    }
  }, [selectedTopicId, getNotesForTopic, excludeJournal]);

  // Debounced search
  useEffect(() => {
    if (!searchInput.trim()) {
      setSearchQuery('');
      return;
    }
    const timeout = setTimeout(() => {
      semanticSearch(searchInput);
      setSearchQuery(searchInput);
    }, 300);
    return () => clearTimeout(timeout);
  }, [searchInput, semanticSearch, setSearchQuery]);

  const handleTopicClick = useCallback(
    (topicId: string) => selectTopic(topicId === selectedTopicId ? null : topicId),
    [selectTopic, selectedTopicId],
  );

  const handleReclassify = useCallback(async () => {
    console.log('[useTopicsData] handleReclassify called');
    console.log('[useTopicsData] Step 1: Initializing embedding service...');
    const ready = await initialize();
    console.log('[useTopicsData] Step 1 result: ready =', ready);
    if (ready) {
      console.log('[useTopicsData] Step 2: Reclassifying all notes...');
      const result = await reclassifyAllNotes({ excludeJournal });
      console.log('[useTopicsData] Step 2 result:', result);
      console.log('[useTopicsData] Step 3: Getting embedding status...');
      await getEmbeddingStatus();
      console.log('[useTopicsData] Step 4: Loading topics...');
      await loadTopics({ excludeJournal });
      console.log('[useTopicsData] Reclassify complete!');
    } else {
      console.error('[useTopicsData] Initialization failed, skipping reclassify');
    }
  }, [initialize, reclassifyAllNotes, getEmbeddingStatus, loadTopics, excludeJournal]);

  const handleCreateTopic = useCallback(
    async (name: string, color: string) => {
      await createTopic({ name: name.trim(), color });
      await loadTopics({ excludeJournal });
    },
    [createTopic, loadTopics, excludeJournal],
  );

  const selectedTopic = useMemo(
    () => topics.find((t) => t.id === selectedTopicId),
    [topics, selectedTopicId],
  );

  return {
    // State
    topics,
    selectedTopicId,
    selectedTopic,
    topicNotes,
    embeddingStatus,
    searchResults,
    searchQuery,
    searchInput,
    classifying,
    error,
    initializing,
    loadingNotes,
    excludeJournal,
    // Actions
    setSearchInput,
    setExcludeJournal,
    handleTopicClick,
    handleReclassify,
    handleCreateTopic,
    selectTopic,
  };
}

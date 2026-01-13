/**
 * Notebook API Hook - React hooks for notebook operations
 *
 * Uses createEntityAPI for base CRUD, extends with notebook-specific operations.
 */

import { useCallback } from 'react';
import { useNotebookStore } from '@/stores/notebookStore';
import { Notebook } from '@/types';
import { NOTEBOOK_COMMANDS } from '@/constants/tauriCommands';
import { createEntityAPI } from './createEntityAPI';
import { notebookAPI } from '@/api';

/**
 * Base CRUD operations from factory
 */
const useNotebookCRUD = createEntityAPI<Notebook>({
  entityName: 'notebook',
  channels: {
    GET_ALL: NOTEBOOK_COMMANDS.GET_ALL,
    CREATE: NOTEBOOK_COMMANDS.CREATE,
    UPDATE: NOTEBOOK_COMMANDS.UPDATE,
    DELETE: NOTEBOOK_COMMANDS.DELETE,
  },
  useStore: () => {
    const store = useNotebookStore();
    return {
      setItems: store.setNotebooks,
      addItem: store.addNotebook,
      updateItem: store.updateNotebook,
      deleteItem: store.deleteNotebook,
      setLoading: store.setLoading,
      setError: store.setError,
    };
  },
  responseKey: 'notebooks',
  logPrefix: '[NotebookAPI]',
});

/**
 * Notebook API hook with CRUD + notebook-specific operations
 */
export function useNotebookAPI() {
  const { loadAll, create, update } = useNotebookCRUD();
  const { setError } = useNotebookStore();

  /**
   * Load notebooks with optional flat mode
   */
  const loadNotebooks = useCallback(
    async (flat = false) => {
      return loadAll({ include_counts: true, flat });
    },
    [loadAll],
  );

  /**
   * Create a new notebook
   */
  const createNotebook = useCallback(
    async (data: { name: string; parent_id?: string; icon?: string; color?: string }) => {
      return create(data as Partial<Notebook>);
    },
    [create],
  );

  /**
   * Update notebook data
   */
  const updateNotebookData = useCallback(
    async (id: string, data: { name?: string; icon?: string; color?: string }) => {
      return update(id, data as Partial<Notebook>);
    },
    [update],
  );

  /**
   * Delete a notebook
   */
  const deleteNotebookById = useCallback(
    async (id: string, deleteNotes = false) => {
      setError(null);
      try {
        const response = await notebookAPI.delete(id, deleteNotes);
        if (response.success) {
          useNotebookStore.getState().deleteNotebook(id);
          return true;
        }
        setError(response.error?.message || 'Failed to delete notebook');
        return false;
      } catch (error) {
        setError(error instanceof Error ? error.message : 'Failed to delete notebook');
        return false;
      }
    },
    [setError],
  );

  /**
   * Move a notebook to a new parent or position
   */
  const moveNotebook = useCallback(
    async (id: string, parentId?: string, position?: number) => {
      setError(null);
      try {
        const response = await notebookAPI.move(id, parentId, position);
        if (response.success) {
          // Reload notebooks to update the tree structure
          await loadNotebooks();
          return true;
        }
        setError(response.error?.message || 'Failed to move notebook');
        return false;
      } catch (error) {
        setError(error instanceof Error ? error.message : 'Failed to move notebook');
        return false;
      }
    },
    [loadNotebooks, setError],
  );

  return {
    loadNotebooks,
    createNotebook,
    updateNotebook: updateNotebookData,
    deleteNotebook: deleteNotebookById,
    moveNotebook,
  };
}

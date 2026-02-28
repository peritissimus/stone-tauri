/**
 * Table Navigation Extension
 *
 * Provides Notion-like keyboard navigation for tables:
 * - Tab: Move to next cell (or create new row if at end)
 * - Shift+Tab: Move to previous cell
 * - Arrow Down in last cell: Exit table, create paragraph below
 * - Cmd+Enter: Exit table from anywhere
 * - Enter in empty last cell: Exit table instead of staying
 */

import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { TextSelection } from '@tiptap/pm/state';
import { logger } from '@/utils/logger';

// Helper: Check if cursor is in a table
function isInTable(state: any): boolean {
  const { $from } = state.selection;
  for (let d = $from.depth; d > 0; d--) {
    if ($from.node(d).type.name === 'table') {
      return true;
    }
  }
  return false;
}

// Helper: Check if cursor is in the last cell of the table
function isInLastCell(state: any): boolean {
  const { $from } = state.selection;

  // Find the table node
  let tableDepth = -1;
  for (let d = $from.depth; d > 0; d--) {
    if ($from.node(d).type.name === 'table') {
      tableDepth = d;
      break;
    }
  }

  if (tableDepth === -1) return false;

  const table = $from.node(tableDepth);
  const rows = table.content.content;
  const lastRow = rows[rows.length - 1];

  if (!lastRow) return false;

  const lastRowCells = lastRow.content.content;
  const lastCell = lastRowCells[lastRowCells.length - 1];

  // Check if cursor's cell is the last cell
  for (let d = $from.depth; d > 0; d--) {
    const node = $from.node(d);
    if (node.type.name === 'tableCell' || node.type.name === 'tableHeader') {
      return node === lastCell;
    }
  }

  return false;
}

// Helper: Check if current cell is empty
function isCurrentCellEmpty(state: any): boolean {
  const { $from } = state.selection;

  for (let d = $from.depth; d > 0; d--) {
    const node = $from.node(d);
    if (node.type.name === 'tableCell' || node.type.name === 'tableHeader') {
      // Cell is empty if it has only one paragraph child that's empty
      return node.content.size <= 2; // Empty paragraph has size 2 (opening + closing)
    }
  }

  return false;
}

// Helper: Exit table and create paragraph below
function exitTable(state: any, dispatch: any): boolean {
  const { $from } = state.selection;

  // Find the table node position
  let tablePos = -1;
  let tableDepth = -1;
  for (let d = $from.depth; d > 0; d--) {
    if ($from.node(d).type.name === 'table') {
      tablePos = $from.before(d);
      tableDepth = d;
      break;
    }
  }

  if (tablePos === -1) return false;

  const table = $from.node(tableDepth);
  const tableEnd = tablePos + table.nodeSize;

  // Check if there's already content after the table
  const $after = state.doc.resolve(tableEnd);
  const hasContentAfter = $after.nodeAfter !== null;

  if (dispatch) {
    const tr = state.tr;

    if (!hasContentAfter) {
      // Create a new paragraph after the table
      const paragraph = state.schema.nodes.paragraph.create();
      tr.insert(tableEnd, paragraph);
    }

    // Move cursor to the paragraph after table
    const newPos = tableEnd + 1; // Position inside the paragraph
    tr.setSelection(TextSelection.create(tr.doc, newPos));
    tr.scrollIntoView();

    dispatch(tr);
  }

  return true;
}

export const TableNavigation = Extension.create({
  name: 'tableNavigation',

  addKeyboardShortcuts() {
    return {
      // Tab: Move to next cell or create new row
      Tab: ({ editor }) => {
        if (!isInTable(editor.state)) return false;

        // Try to go to next cell first
        if (editor.commands.goToNextCell()) {
          return true;
        }

        // If no next cell, we're at the end - add a new row
        if (editor.commands.addRowAfter && editor.commands.addRowAfter()) {
          // Move to first cell of new row
          editor.commands.goToNextCell();
          return true;
        }

        return false;
      },

      // Shift+Tab: Move to previous cell
      'Shift-Tab': ({ editor }) => {
        if (!isInTable(editor.state)) return false;
        return editor.commands.goToPreviousCell();
      },

      // Cmd/Ctrl+Enter: Exit table from anywhere
      'Mod-Enter': ({ editor }) => {
        if (!isInTable(editor.state)) return false;

        logger.info('[TableNavigation] Cmd+Enter - exiting table');
        return exitTable(editor.state, editor.view.dispatch);
      },

      // Arrow Down: Exit table if in last cell
      ArrowDown: ({ editor }) => {
        if (!isInTable(editor.state)) return false;

        const { $from } = editor.state.selection;
        const cell = $from.node($from.depth - 1);

        // Check if we're at the end of the cell content
        const endOfCell = $from.parentOffset === cell.content.size;

        if (endOfCell && isInLastCell(editor.state)) {
          logger.info('[TableNavigation] Arrow down in last cell - exiting table');
          return exitTable(editor.state, editor.view.dispatch);
        }

        // Default behavior - let ProseMirror handle it
        return false;
      },

      // Enter: If in empty last cell, exit table
      Enter: ({ editor }) => {
        if (!isInTable(editor.state)) return false;

        if (isInLastCell(editor.state) && isCurrentCellEmpty(editor.state)) {
          logger.info('[TableNavigation] Enter in empty last cell - exiting table');
          return exitTable(editor.state, editor.view.dispatch);
        }

        // Default behavior - create new paragraph in cell
        return false;
      },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey('tableNavigation'),
        props: {
          // Optional: Add visual feedback when in table
          decorations(_state) {
            // Could add decorations here if needed
            return null;
          },
        },
      }),
    ];
  },
});

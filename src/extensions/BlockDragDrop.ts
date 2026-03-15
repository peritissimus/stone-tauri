/**
 * Block Drag & Drop Extension
 *
 * Enables Notion-like drag and drop reordering of blocks.
 * Works with FloatingBlockMenu drag handles.
 */

import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Slice, Fragment } from '@tiptap/pm/model';
import { TextSelection } from '@tiptap/pm/state';
import { logger } from '@/utils/logger';

// Store drag state globally
let draggedNodePos: number | null = null;
let draggedNode: any = null;

export const BlockDragDrop = Extension.create({
  name: 'blockDragDrop',

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey('blockDragDrop'),
        props: {
          handleDOMEvents: {
            dragstart(view, event) {
              const target = event.target as HTMLElement;

              // Only handle drags from our drag handles
              const dragHandle = target.closest('[data-drag-handle]');
              if (!dragHandle) {
                return false;
              }

              logger.info('[BlockDragDrop] Drag started');

              // Find the block being dragged
              const { selection } = view.state;
              const { $from } = selection;

              // Find the top-level block node
              let blockPos = $from.pos;
              let blockNode = null;

              for (let d = $from.depth; d >= 0; d--) {
                const node = $from.node(d);
                if (node.type.name === 'doc') continue;

                // We want direct children of doc
                const parent = d > 0 ? $from.node(d - 1) : null;
                if (parent && parent.type.name === 'doc') {
                  blockPos = $from.before(d);
                  blockNode = node;
                  break;
                }
              }

              if (!blockNode) {
                logger.warn('[BlockDragDrop] No block node found');
                return false;
              }

              logger.info('[BlockDragDrop] Dragging block:', blockNode.type.name, 'at pos:', blockPos);

              // Store the dragged node info
              draggedNodePos = blockPos;
              draggedNode = blockNode;

              // Set drag data
              event.dataTransfer!.effectAllowed = 'move';
              event.dataTransfer!.setData('text/plain', ''); // Required for Firefox

              // Add drag image (optional - browser will use default)
              if (typeof event.dataTransfer!.setDragImage === 'function') {
                const blockDom = view.nodeDOM(blockPos);
                if (blockDom instanceof HTMLElement) {
                  event.dataTransfer!.setDragImage(blockDom, 0, 0);
                }
              }

              return true;
            },

            dragover(_view, event) {
              // Must prevent default to allow drop
              if (draggedNodePos !== null) {
                event.preventDefault();
                event.dataTransfer!.dropEffect = 'move';
                return true;
              }
              return false;
            },

            drop(view, event) {
              if (draggedNodePos === null || !draggedNode) {
                return false;
              }

              logger.info('[BlockDragDrop] Drop triggered');

              event.preventDefault();
              event.stopPropagation();

              // Find drop position
              const dropPos = view.posAtCoords({
                left: event.clientX,
                top: event.clientY,
              });

              if (!dropPos) {
                logger.warn('[BlockDragDrop] Could not find drop position');
                draggedNodePos = null;
                draggedNode = null;
                return true;
              }

              logger.info('[BlockDragDrop] Drop position:', dropPos.pos);

              // Find the target block to drop near
              const $dropPos = view.state.doc.resolve(dropPos.pos);
              let targetPos = dropPos.pos;

              // Find the block at drop position
              for (let d = $dropPos.depth; d >= 0; d--) {
                const node = $dropPos.node(d);
                if (node.type.name === 'doc') continue;

                const parent = d > 0 ? $dropPos.node(d - 1) : null;
                if (parent && parent.type.name === 'doc') {
                  // Determine if we're dropping before or after this block
                  const blockStart = $dropPos.before(d);
                  const blockEnd = $dropPos.after(d);
                  const midpoint = (blockStart + blockEnd) / 2;

                  if (dropPos.pos < midpoint) {
                    // Drop before
                    targetPos = blockStart;
                  } else {
                    // Drop after
                    targetPos = blockEnd;
                  }
                  break;
                }
              }

              logger.info('[BlockDragDrop] Calculated target position:', targetPos);

              // Don't do anything if dropping in the same place
              const nodeSize = draggedNode.nodeSize;
              if (targetPos >= draggedNodePos && targetPos <= draggedNodePos + nodeSize) {
                logger.info('[BlockDragDrop] Dropping in same place, no-op');
                draggedNodePos = null;
                draggedNode = null;
                return true;
              }

              // Create transaction to move the node
              const tr = view.state.tr;

              // Delete from original position
              tr.delete(draggedNodePos, draggedNodePos + nodeSize);

              // Adjust target position if needed (if deleting before target)
              let adjustedTargetPos = targetPos;
              if (targetPos > draggedNodePos) {
                adjustedTargetPos -= nodeSize;
              }

              logger.info('[BlockDragDrop] Final insert position:', adjustedTargetPos);

              // Insert at new position
              const slice = new Slice(Fragment.from(draggedNode), 0, 0);
              tr.insert(adjustedTargetPos, slice.content);

              // Set selection to the moved block
              const newPos = adjustedTargetPos + 1; // Inside the moved block
              tr.setSelection(TextSelection.near(tr.doc.resolve(newPos)));

              // Dispatch transaction
              view.dispatch(tr);

              logger.info('[BlockDragDrop] Block moved successfully!');

              // Clear drag state
              draggedNodePos = null;
              draggedNode = null;

              return true;
            },

            dragend() {
              // Clear drag state on drag end (even if drop didn't happen)
              draggedNodePos = null;
              draggedNode = null;
              return false;
            },
          },
        },
      }),
    ];
  },
});

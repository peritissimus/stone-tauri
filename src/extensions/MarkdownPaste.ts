/**
 * Markdown Paste Extension
 *
 * Handles pasting markdown content and parsing it into proper ProseMirror nodes.
 * This extension intercepts paste events and converts markdown syntax to formatted blocks.
 * Also handles image paste events (images are ignored and left to useImageUpload).
 */

import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Slice } from '@tiptap/pm/model';
import { parseMarkdown } from '@/lib/markdownParser';
import { logger } from '@/utils/logger';

export const MarkdownPaste = Extension.create({
  name: 'markdownPaste',

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey('markdownPaste'),
        props: {
          handlePaste: (view, event, _slice) => {
            // Only handle plain text paste (not rich text from other editors)
            const text = event.clipboardData?.getData('text/plain');

            if (!text) {
              return false; // Let default handler deal with it
            }

            // Check if this is likely markdown content
            // (has markdown syntax like #, *, -, `, etc.)
            const hasMarkdownSyntax = /^#{1,6}\s|^\*\*|^[-*+]\s|^```|^\d+\.\s|^\[\[|\[\d{2}:\d{2}\]/m.test(text);

            if (!hasMarkdownSyntax && text.length < 100) {
              // Short plain text without markdown syntax - use default paste
              return false;
            }

            try {
              logger.info('[MarkdownPaste] Parsing markdown paste (', text.length, 'chars)');

              // Parse markdown to ProseMirror JSON
              const doc = parseMarkdown(text);

              // Convert to ProseMirror node using editor's schema
              const schema = view.state.schema;
              const content = schema.nodeFromJSON(doc);

              // Create a slice from the parsed content
              const parsedSlice = new Slice(content.content, 0, 0);

              // Create a transaction to insert the parsed content
              const tr = view.state.tr.replaceSelection(parsedSlice);

              // Dispatch the transaction to update the editor
              view.dispatch(tr);

              logger.info('[MarkdownPaste] ✓ Paste successful');

              // Prevent default paste behavior
              return true;
            } catch (error) {
              logger.error('[MarkdownPaste] Failed to parse:', error);
              // Fall back to default paste behavior
              return false;
            }
          },
        },
      }),
    ];
  },
});

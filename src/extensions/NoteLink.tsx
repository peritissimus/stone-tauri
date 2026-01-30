/**
 * NoteLink Extension - [[note name]] linking with autocomplete
 *
 * This extension provides:
 * 1. A custom node type for [[note name]] syntax
 * 2. Autocomplete when typing [[
 * 3. Clickable links that navigate to the linked note (via ProseMirror plugin)
 *
 * Note: Uses native DOM rendering (renderHTML) instead of React NodeView
 * for better clipboard serialization and copy/paste support.
 */

import { Node, mergeAttributes } from '@tiptap/core';
import { ReactRenderer } from '@tiptap/react';
import Suggestion, { SuggestionProps, SuggestionKeyDownProps } from '@tiptap/suggestion';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import tippy, { Instance as TippyInstance } from 'tippy.js';

// Unique plugin key for note link suggestion
const noteLinkSuggestionPluginKey = new PluginKey('noteLinkSuggestion');
// Plugin key for note link click handling
const noteLinkClickPluginKey = new PluginKey('noteLinkClick');

import {
  NoteLinkMenu,
  NoteLinkMenuRef,
  NoteLinkItem,
} from '../components/features/Editor/NoteLinkMenu';

export interface NoteLinkOptions {
  /**
   * Callback to fetch notes for autocomplete
   */
  fetchNotes: (query: string) => Promise<NoteLinkItem[]>;

  /**
   * Callback when a note link is clicked
   */
  onNoteLinkClick?: (noteId: string, title: string) => void;

  /**
   * HTML attributes for the node
   */
  HTMLAttributes: Record<string, any>;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    noteLink: {
      /**
       * Insert a note link
       */
      insertNoteLink: (attrs: { noteId: string; title: string }) => ReturnType;
    };
  }
}

export const NoteLink = Node.create<NoteLinkOptions>({
  name: 'noteLink',

  group: 'inline',

  inline: true,

  selectable: true,

  atom: true, // Treat as a single unit (can't edit inside)

  addOptions() {
    return {
      fetchNotes: async () => [],
      onNoteLinkClick: undefined,
      HTMLAttributes: {},
    };
  },

  addAttributes() {
    return {
      noteId: {
        default: null,
        parseHTML: (element) => element.dataset.noteId,
        renderHTML: (attributes) => ({
          'data-note-id': attributes.noteId,
        }),
      },
      title: {
        default: null,
        parseHTML: (element) => element.dataset.title,
        renderHTML: (attributes) => ({
          'data-title': attributes.title,
        }),
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-type="note-link"]',
      },
    ];
  },

  renderHTML({ node, HTMLAttributes }) {
    const title = node.attrs.title || 'Unknown';
    const noteId = node.attrs.noteId;

    // Render as a styled span with link icon (using CSS for the icon)
    return [
      'span',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        'data-type': 'note-link',
        'data-note-id': noteId,
        'data-title': title,
        class: 'note-link-badge',
      }),
      title,
    ];
  },

  // Note: No addNodeView() - using native DOM rendering for better clipboard support

  addCommands() {
    return {
      insertNoteLink:
        (attrs) =>
        ({ commands }) => {
          return commands.insertContent({
            type: this.name,
            attrs,
          });
        },
    };
  },

  addProseMirrorPlugins() {
    const { fetchNotes } = this.options;

    return [
      // Click handling plugin for note link navigation
      new Plugin({
        key: noteLinkClickPluginKey,
        props: {
          handleClick(_view, _pos, event) {
            const target = event.target as HTMLElement;

            // Check if clicked on a note link
            if (target.dataset.type === 'note-link' || target.closest('[data-type="note-link"]')) {
              const linkElement = target.dataset.type === 'note-link'
                ? target
                : target.closest('[data-type="note-link"]') as HTMLElement;

              if (!linkElement) return false;

              const noteId = linkElement.dataset.noteId;
              const title = linkElement.dataset.title;

              // Dispatch a custom event that can be caught by the editor container
              const customEvent = new CustomEvent('note-link-click', {
                bubbles: true,
                detail: { noteId, title },
              });
              linkElement.dispatchEvent(customEvent);

              return true;
            }

            return false;
          },
        },
      }),

      // Autocomplete suggestion plugin
      Suggestion({
        pluginKey: noteLinkSuggestionPluginKey,
        editor: this.editor,
        char: '[[',
        allowSpaces: true,
        startOfLine: false,

        items: async ({ query }: { query: string }) => {
          // Fetch notes matching the query
          const notes = await fetchNotes(query);
          return notes;
        },

        command: ({ editor, range, props }: any) => {
          const item = props as NoteLinkItem;

          // Delete the trigger and query text
          editor
            .chain()
            .focus()
            .deleteRange(range)
            .insertContent({
              type: 'noteLink',
              attrs: {
                noteId: item.id,
                title: item.title,
              },
            })
            .run();
        },

        render: () => {
          let component: ReactRenderer<NoteLinkMenuRef>;
          let popup: TippyInstance[];

          return {
            onStart: (props: SuggestionProps<NoteLinkItem>) => {
              component = new ReactRenderer(NoteLinkMenu, {
                props: {
                  ...props,
                  items: props.items || [],
                },
                editor: props.editor,
              });

              if (!props.clientRect) {
                return;
              }

              popup = tippy('body', {
                getReferenceClientRect: props.clientRect as () => DOMRect,
                appendTo: () => document.body,
                content: component.element,
                showOnCreate: true,
                interactive: true,
                trigger: 'manual',
                placement: 'bottom-start',
                maxWidth: 'none',
                offset: [0, 8],
                theme: 'note-link',
              });
            },

            onUpdate(props: SuggestionProps<NoteLinkItem>) {
              component.updateProps({
                ...props,
                items: props.items || [],
              });

              if (!props.clientRect) {
                return;
              }

              popup[0].setProps({
                getReferenceClientRect: props.clientRect as () => DOMRect,
              });
            },

            onKeyDown(props: SuggestionKeyDownProps) {
              if (props.event.key === 'Escape') {
                popup[0].hide();
                return true;
              }

              return component.ref?.onKeyDown(props) || false;
            },

            onExit() {
              popup[0].destroy();
              component.destroy();
            },
          };
        },
      }),
    ];
  },
});

export default NoteLink;

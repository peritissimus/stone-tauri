/**
 * Markdown Parser using prosemirror-markdown
 *
 * Converts markdown text directly to ProseMirror document structure,
 * bypassing the previous markdown → HTML → ProseMirror conversion.
 *
 * Custom plugins handle:
 * - Task markers (TODO, DOING, DONE, etc.)
 * - Timestamps [HH:MM]
 * - Note links [[title]]
 */

import MarkdownIt from 'markdown-it';
import type StateCore from 'markdown-it/lib/rules_core/state_core.mjs';
import type Token from 'markdown-it/lib/token.mjs';
import type { JSONContent } from '@tiptap/core';

// Task marker states
const TASK_MARKERS = ['TODO', 'DOING', 'DONE', 'WAITING', 'HOLD', 'CANCELED', 'CANCELLED', 'IDEA'];
const TASK_MARKER_PATTERN = new RegExp(`\\b(${TASK_MARKERS.join('|')})\\b`, 'g');

// Timestamp pattern [HH:MM]
const TIMESTAMP_PATTERN = /\[([01]?[0-9]|2[0-3]):([0-5][0-9])\]/g;

// Note link pattern [[title]]
const NOTE_LINK_PATTERN = /\[\[([^\]]+)\]\]/g;

/**
 * markdown-it plugin for task markers (TODO, DOING, DONE, etc.)
 */
function taskMarkerPlugin(md: MarkdownIt) {
  // Process inline tokens to find and replace task markers
  md.core.ruler.after('inline', 'task_marker', (state: StateCore) => {
    for (const blockToken of state.tokens) {
      if (blockToken.type !== 'inline' || !blockToken.children) continue;

      const newChildren: Token[] = [];

      for (const token of blockToken.children) {
        if (token.type !== 'text' || !token.content) {
          newChildren.push(token);
          continue;
        }

        // Split text by task markers
        let lastIndex = 0;
        let match: RegExpExecArray | null;
        TASK_MARKER_PATTERN.lastIndex = 0;

        while ((match = TASK_MARKER_PATTERN.exec(token.content)) !== null) {
          // Add text before match
          if (match.index > lastIndex) {
            const textToken = new state.Token('text', '', 0);
            textToken.content = token.content.slice(lastIndex, match.index);
            newChildren.push(textToken);
          }

          // Add task marker token
          const markerToken = new state.Token('task_marker', '', 0);
          const markerText = match[1].toUpperCase();
          // Normalize CANCELLED to CANCELED
          markerToken.meta = {
            state: markerText === 'CANCELLED' ? 'canceled' : markerText.toLowerCase(),
          };
          newChildren.push(markerToken);

          lastIndex = match.index + match[0].length;
        }

        // If no matches were found, keep original token
        if (lastIndex === 0) {
          newChildren.push(token);
        } else if (lastIndex < token.content.length) {
          // Add remaining text after last match
          const textToken = new state.Token('text', '', 0);
          textToken.content = token.content.slice(lastIndex);
          newChildren.push(textToken);
        }
      }

      blockToken.children = newChildren;
    }
    return true;
  });
}

/**
 * markdown-it plugin for timestamps [HH:MM]
 */
function timestampPlugin(md: MarkdownIt) {
  md.core.ruler.after('inline', 'timestamp', (state: StateCore) => {
    for (const blockToken of state.tokens) {
      if (blockToken.type !== 'inline' || !blockToken.children) continue;

      const newChildren: Token[] = [];

      for (const token of blockToken.children) {
        if (token.type !== 'text' || !token.content) {
          newChildren.push(token);
          continue;
        }

        // Split text by timestamps
        let lastIndex = 0;
        let match: RegExpExecArray | null;
        TIMESTAMP_PATTERN.lastIndex = 0;

        while ((match = TIMESTAMP_PATTERN.exec(token.content)) !== null) {
          // Add text before match
          if (match.index > lastIndex) {
            const textToken = new state.Token('text', '', 0);
            textToken.content = token.content.slice(lastIndex, match.index);
            newChildren.push(textToken);
          }

          // Add timestamp token
          const timestampToken = new state.Token('timestamp', '', 0);
          const hours = match[1].padStart(2, '0');
          const minutes = match[2];
          timestampToken.meta = { time: `${hours}:${minutes}` };
          newChildren.push(timestampToken);

          lastIndex = match.index + match[0].length;
        }

        // If no matches were found, keep original token
        if (lastIndex === 0) {
          newChildren.push(token);
        } else if (lastIndex < token.content.length) {
          // Add remaining text after last match
          const textToken = new state.Token('text', '', 0);
          textToken.content = token.content.slice(lastIndex);
          newChildren.push(textToken);
        }
      }

      blockToken.children = newChildren;
    }
    return true;
  });
}

/**
 * markdown-it plugin for note links [[title]]
 */
function noteLinkPlugin(md: MarkdownIt) {
  md.core.ruler.after('inline', 'note_link', (state: StateCore) => {
    for (const blockToken of state.tokens) {
      if (blockToken.type !== 'inline' || !blockToken.children) continue;

      const newChildren: Token[] = [];

      for (const token of blockToken.children) {
        if (token.type !== 'text' || !token.content) {
          newChildren.push(token);
          continue;
        }

        // Split text by note links
        let lastIndex = 0;
        let match: RegExpExecArray | null;
        NOTE_LINK_PATTERN.lastIndex = 0;

        while ((match = NOTE_LINK_PATTERN.exec(token.content)) !== null) {
          // Add text before match
          if (match.index > lastIndex) {
            const textToken = new state.Token('text', '', 0);
            textToken.content = token.content.slice(lastIndex, match.index);
            newChildren.push(textToken);
          }

          // Add note link token
          const noteLinkToken = new state.Token('note_link', '', 0);
          noteLinkToken.meta = { title: match[1] };
          newChildren.push(noteLinkToken);

          lastIndex = match.index + match[0].length;
        }

        // If no matches were found, keep original token
        if (lastIndex === 0) {
          newChildren.push(token);
        } else if (lastIndex < token.content.length) {
          // Add remaining text after last match
          const textToken = new state.Token('text', '', 0);
          textToken.content = token.content.slice(lastIndex);
          newChildren.push(textToken);
        }
      }

      blockToken.children = newChildren;
    }
    return true;
  });
}

// Create markdown-it instance with custom plugins
const md = new MarkdownIt('default', { html: false, breaks: true })
  .use(taskMarkerPlugin)
  .use(timestampPlugin)
  .use(noteLinkPlugin);

// Token type definitions for mapping markdown tokens to ProseMirror nodes
interface TokenSpec {
  node?: string;
  block?: string;
  mark?: string;
  getAttrs?: (token: Token) => Record<string, unknown> | null;
  noCloseToken?: boolean;
  ignore?: boolean;
  noClose?: boolean; // If true, the corresponding _close token should NOT close a block
}

/**
 * Token mappings from markdown-it tokens to ProseMirror nodes/marks
 */
const defaultTokens: Record<string, TokenSpec> = {
  // Custom tokens
  task_marker: {
    node: 'taskMarker',
    getAttrs: (token) => ({ state: token.meta?.state || 'todo' }),
  },
  timestamp: {
    node: 'timestamp',
    getAttrs: (token) => ({ time: token.meta?.time || '00:00' }),
  },
  note_link: {
    node: 'noteLink',
    getAttrs: (token) => ({
      title: token.meta?.title || 'Unknown',
      noteId: null, // Will be resolved later if needed
    }),
  },

  // Block tokens
  paragraph_open: { block: 'paragraph' },
  blockquote_open: { block: 'blockquote' },
  bullet_list_open: { block: 'bulletList' },
  ordered_list_open: { block: 'orderedList', getAttrs: (token) => ({ start: token.attrGet('start') || 1 }) },
  list_item_open: { block: 'listItem' },
  heading_open: {
    block: 'heading',
    getAttrs: (token) => ({ level: parseInt(token.tag.slice(1), 10) }),
  },
  code_block: {
    block: 'codeBlock',
    noCloseToken: true,
    getAttrs: (token) => ({ language: token.info || null }),
  },
  fence: {
    block: 'codeBlock',
    noCloseToken: true,
    getAttrs: (token) => ({ language: token.info || null }),
  },
  hr: { node: 'horizontalRule' },

  // Table tokens
  table_open: { block: 'table' },
  thead_open: { ignore: true, noClose: true },
  thead_close: { ignore: true },
  tbody_open: { ignore: true, noClose: true },
  tbody_close: { ignore: true },
  tr_open: { block: 'tableRow' },
  th_open: { block: 'tableHeader' },
  td_open: { block: 'tableCell' },

  // Inline tokens
  image: {
    node: 'image',
    getAttrs: (token) => ({
      src: token.attrGet('src'),
      alt: token.children?.[0]?.content || token.content || '',
      title: token.attrGet('title'),
    }),
  },
  hardbreak: { node: 'hardBreak' },
  softbreak: { node: 'hardBreak' },

  // Mark tokens
  em_open: { mark: 'italic' },
  strong_open: { mark: 'bold' },
  s_open: { mark: 'strike' },
  code_inline: { mark: 'code', noCloseToken: true },
  link_open: {
    mark: 'link',
    getAttrs: (token) => ({
      href: token.attrGet('href'),
      title: token.attrGet('title'),
    }),
  },
};

/**
 * Parse markdown string into ProseMirror document JSON
 * This function creates a document structure compatible with TipTap's setContent
 *
 * @param markdown - The markdown string to parse
 * @param _schema - The ProseMirror schema (reserved for future validation use)
 * @returns A JSONContent object compatible with TipTap's setContent
 */
export function parseMarkdown(markdown: string, _schema?: unknown): JSONContent {
  const tokens = md.parse(markdown, {});
  const doc: JSONContent = { type: 'doc', content: [] };
  const stack: JSONContent[] = [doc];
  const markStack: { type: string; attrs?: Record<string, unknown> }[] = [];

  function top() {
    return stack[stack.length - 1];
  }

  function addNode(
    type: string,
    attrs?: Record<string, unknown>,
    content?: JSONContent[],
    marks?: { type: string; attrs?: Record<string, unknown> }[]
  ) {
    const node: JSONContent = { type };
    if (attrs && Object.keys(attrs).length > 0) node.attrs = attrs;
    if (content && content.length > 0) node.content = content;
    if (marks && marks.length > 0) node.marks = marks;
    top().content!.push(node);
  }

  function addText(text: string) {
    if (!text) return;
    const node: JSONContent = { type: 'text', text };
    if (markStack.length > 0) {
      node.marks = markStack.map((m) => ({ type: m.type, ...(m.attrs ? { attrs: m.attrs } : {}) }));
    }
    top().content!.push(node);
  }

  function openBlock(type: string, attrs?: Record<string, unknown>) {
    const block: JSONContent = {
      type,
      content: [],
    };
    if (attrs && Object.keys(attrs).length > 0) block.attrs = attrs;
    stack.push(block);
  }

  function closeBlock() {
    const block = stack.pop();
    if (block && stack.length > 0) {
      const result: JSONContent = { type: block.type };
      if (block.attrs && Object.keys(block.attrs).length > 0) result.attrs = block.attrs;
      if (block.content && block.content.length > 0) result.content = block.content;
      top().content!.push(result);
    }
  }

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i];
    const spec = defaultTokens[token.type];

    // Handle closing tokens first - they may not have specs but need to close blocks
    // Close tokens have nesting === -1 and typically end with _close
    if (token.nesting === -1) {
      if (token.type.endsWith('_close')) {
        // Check if the corresponding _open token has noClose flag
        const openTokenType = token.type.replace('_close', '_open');
        const openSpec = defaultTokens[openTokenType];
        if (!openSpec?.noClose) {
          closeBlock();
        }
      }
      continue;
    }

    if (!spec) {
      // Handle text tokens
      if (token.type === 'text') {
        addText(token.content);
      } else if (token.type === 'inline' && token.children) {
        // Process inline children
        for (const child of token.children) {
          const childSpec = defaultTokens[child.type];
          if (childSpec?.node) {
            // Inline node (image, hardbreak, task_marker, etc.)
            const attrs = childSpec.getAttrs?.(child) || {};
            addNode(childSpec.node, attrs, undefined, markStack.length > 0 ? [...markStack] : undefined);
          } else if (childSpec?.mark) {
            // Mark open
            const attrs = childSpec.getAttrs?.(child) || {};
            markStack.push({ type: childSpec.mark, attrs: Object.keys(attrs).length > 0 ? attrs : undefined });
            if (childSpec.noCloseToken && child.content) {
              addText(child.content);
              markStack.pop();
            }
          } else if (child.type.endsWith('_close')) {
            // Mark close
            markStack.pop();
          } else if (child.type === 'text') {
            addText(child.content);
          }
        }
      }
      continue;
    }

    if (spec.ignore) continue;

    if (spec.node) {
      // Self-closing node
      const attrs = spec.getAttrs?.(token) || {};
      addNode(spec.node, attrs);
    } else if (spec.block) {
      if (token.nesting === 1) {
        // Opening tag
        const attrs = spec.getAttrs?.(token) || {};
        openBlock(spec.block, attrs);
      } else if (spec.noCloseToken) {
        // Self-closing block (code_block, fence)
        const attrs = spec.getAttrs?.(token) || {};
        addNode(spec.block, attrs, [{ type: 'text', text: token.content }]);
      }
    } else if (spec.mark) {
      if (token.nesting === 1) {
        const attrs = spec.getAttrs?.(token) || {};
        markStack.push({ type: spec.mark, attrs: Object.keys(attrs).length > 0 ? attrs : undefined });
      } else if (spec.noCloseToken) {
        const attrs = spec.getAttrs?.(token) || {};
        markStack.push({ type: spec.mark, attrs: Object.keys(attrs).length > 0 ? attrs : undefined });
        addText(token.content);
        markStack.pop();
      }
    }
  }

  // Close any remaining blocks
  while (stack.length > 1) {
    closeBlock();
  }

  return doc;
}

export { md as markdownIt };

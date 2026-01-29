/**
 * TaskMarker Extension - Inline task state markers (TODO, DOING, DONE, etc.)
 *
 * This extension provides:
 * 1. A custom inline node type for task markers
 * 2. Automatic detection when typing task keywords at line start
 * 3. Clickable badges that cycle through states
 * 4. Styled badge display via CSS
 */

import { Node, mergeAttributes, InputRule } from '@tiptap/core';
import { NodeViewWrapper, NodeViewProps, ReactNodeViewRenderer } from '@tiptap/react';

export interface TaskMarkerOptions {
  HTMLAttributes: Record<string, unknown>;
  states: TaskState[];
  defaultState: string;
}

export interface TaskState {
  value: string;
  label: string;
  shortLabel?: string;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    taskMarker: {
      /**
       * Insert a task marker with a specific state
       */
      insertTaskMarker: (state: string) => ReturnType;
      /**
       * Cycle task marker to next state
       */
      cycleTaskMarker: () => ReturnType;
    };
  }
}

// Default task states (Logseq-inspired)
const DEFAULT_STATES: TaskState[] = [
  { value: 'todo', label: 'TODO' },
  { value: 'doing', label: 'DOING' },
  { value: 'done', label: 'DONE' },
  { value: 'waiting', label: 'WAITING', shortLabel: 'WAIT' },
  { value: 'hold', label: 'HOLD' },
  { value: 'canceled', label: 'CANCELED', shortLabel: 'CAN' },
  { value: 'idea', label: 'IDEA' },
];

// Regex to match task markers at word boundary followed by space
// Matches: TODO, DOING, DONE, WAITING, HOLD, CANCELED, CANCELLED, IDEA
const TASK_MARKER_INPUT_REGEX = /^(TODO|DOING|DONE|WAITING|HOLD|CANCELED|CANCELLED|IDEA)\s$/i;

// React component for the task marker badge
function TaskMarkerView({ node, updateAttributes }: NodeViewProps) {
  const state = (node.attrs.state as string) || 'todo';

  const cycleState = () => {
    const states = DEFAULT_STATES;
    const currentIndex = states.findIndex((s) => s.value === state);
    const nextIndex = (currentIndex + 1) % states.length;
    updateAttributes({ state: states[nextIndex].value });
  };

  const getLabel = () => {
    const stateConfig = DEFAULT_STATES.find((s) => s.value === state);
    return stateConfig?.shortLabel || stateConfig?.label || state.toUpperCase();
  };

  return (
    <NodeViewWrapper as="span" className="task-marker-wrapper">
      <span
        data-type="task-marker"
        data-state={state}
        className="task-marker-badge"
        onClick={cycleState}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            cycleState();
          }
        }}
        role="button"
        tabIndex={0}
        contentEditable={false}
      >
        {getLabel()}
      </span>
    </NodeViewWrapper>
  );
}

export const TaskMarker = Node.create<TaskMarkerOptions>({
  name: 'taskMarker',

  group: 'inline',

  inline: true,

  selectable: true,

  atom: true, // Treat as a single unit

  addOptions() {
    return {
      HTMLAttributes: {},
      states: DEFAULT_STATES,
      defaultState: 'todo',
    };
  },

  addAttributes() {
    return {
      state: {
        default: 'todo',
        parseHTML: (element) => element.dataset.state || 'todo',
        renderHTML: (attributes) => ({
          'data-state': attributes.state,
        }),
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-type="task-marker"]',
      },
    ];
  },

  renderHTML({ node, HTMLAttributes }) {
    const state = node.attrs.state as string;
    const stateConfig = this.options.states.find((s) => s.value === state);
    const label = stateConfig?.shortLabel || stateConfig?.label || state.toUpperCase();

    return [
      'span',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        'data-type': 'task-marker',
        'data-state': state,
        class: 'task-marker-badge',
      }),
      label,
    ];
  },

  addNodeView() {
    return ReactNodeViewRenderer(TaskMarkerView);
  },

  addCommands() {
    return {
      insertTaskMarker:
        (state: string) =>
        ({ commands }) => {
          return commands.insertContent({
            type: this.name,
            attrs: { state },
          });
        },
      cycleTaskMarker:
        () =>
        ({ state, commands }) => {
          const { selection } = state;
          const node = state.doc.nodeAt(selection.from);

          if (node?.type.name === this.name) {
            const currentState = node.attrs.state as string;
            const states = this.options.states;
            const currentIndex = states.findIndex((s) => s.value === currentState);
            const nextIndex = (currentIndex + 1) % states.length;

            return commands.updateAttributes(this.name, {
              state: states[nextIndex].value,
            });
          }

          return false;
        },
    };
  },

  addInputRules() {
    return [
      new InputRule({
        find: TASK_MARKER_INPUT_REGEX,
        handler: ({ state, range, match }) => {
          const markerText = match[1].toUpperCase();
          // Normalize CANCELLED to CANCELED
          const normalizedMarker = markerText === 'CANCELLED' ? 'CANCELED' : markerText;
          const taskState = normalizedMarker.toLowerCase();

          const { tr } = state;
          const start = range.from;
          const end = range.to;

          tr.replaceWith(start, end, this.type.create({ state: taskState }));
          // Add a space after the marker for continued typing
          tr.insertText(' ');
        },
      }),
    ];
  },
});

export default TaskMarker;

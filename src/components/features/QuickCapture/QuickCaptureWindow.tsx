/**
 * QuickCaptureWindow - Floating window for quick journal capture
 *
 * Optimized for speed: closes immediately on submit, save happens in background
 */

import React, { useState, useRef, useEffect } from 'react';
import { useQuickCaptureAPI } from '@/hooks/useQuickCaptureAPI';
import { quickCaptureAPI } from '@/api/quickCaptureAPI';

const DRAFT_KEY = 'quick-capture-draft';

export function QuickCaptureWindow() {
  const { appendToJournal } = useQuickCaptureAPI();
  const [text, setText] = useState(() => {
    // Restore draft on mount
    return localStorage.getItem(DRAFT_KEY) || '';
  });
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const closeWindow = async () => {
    console.log('[QuickCapture] Attempting to hide window via backend...');
    try {
      const response = await quickCaptureAPI.hide();
      if (response.success) {
        console.log('[QuickCapture] Window hidden successfully');
      } else {
        console.error('[QuickCapture] Failed to hide window:', response.error);
      }
    } catch (err) {
      console.error('[QuickCapture] Failed to hide window:', err);
    }
  };

  // Auto-focus immediately
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  // Close when the window loses focus
  useEffect(() => {
    const handleBlur = () => {
      console.log('[QuickCapture] Window blur detected, hiding');
      void closeWindow();
    };

    window.addEventListener('blur', handleBlur);
    return () => window.removeEventListener('blur', handleBlur);
  }, []);

  // Save draft on text change (debounced naturally by React state)
  useEffect(() => {
    if (text.trim()) {
      localStorage.setItem(DRAFT_KEY, text);
    } else {
      localStorage.removeItem(DRAFT_KEY);
    }
  }, [text]);

  const handleSubmit = async () => {
    console.log('[QuickCapture] handleSubmit called with text:', text);
    const trimmedText = text.trim();
    if (!trimmedText) {
      console.log('[QuickCapture] No text to submit, returning');
      return;
    }

    // Clear the text state first
    setText('');

    // Clear draft from localStorage
    localStorage.removeItem(DRAFT_KEY);

    // Close window immediately
    await closeWindow();

    // Save in background - file watcher will auto-refresh main window
    console.log('[QuickCapture] Saving to journal:', trimmedText);
    appendToJournal(trimmedText).catch((err) => {
      // If save fails, restore draft so user doesn't lose content
      console.error('[QuickCapture] Save failed:', err);
      localStorage.setItem(DRAFT_KEY, trimmedText);
    });
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    console.log('[QuickCapture] Key pressed:', e.key, 'metaKey:', e.metaKey, 'ctrlKey:', e.ctrlKey);

    // Cmd/Ctrl+Enter to save
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      console.log('[QuickCapture] Cmd/Ctrl+Enter detected, calling handleSubmit');
      e.preventDefault();
      void handleSubmit();
    }
    if (e.key === 'Escape') {
      console.log('[QuickCapture] Escape pressed, closing window');
      void closeWindow();
    }
  };

  return (
    <div className="h-screen w-screen p-2">
      <textarea
        ref={textareaRef}
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="What's on your mind? (Cmd+Enter to save)"
        autoFocus
        rows={3}
        className="w-full h-full px-4 py-3 text-sm bg-background/80 backdrop-blur-xl rounded-xl border-none outline-none resize-none placeholder:text-xs placeholder:text-muted-foreground/30"
      />
    </div>
  );
}

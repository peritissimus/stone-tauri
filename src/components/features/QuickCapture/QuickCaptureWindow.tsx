/**
 * QuickCaptureWindow - Floating window for quick journal capture
 *
 * Optimized for speed: closes immediately on submit, save happens in background
 */

import React, { useState, useRef, useEffect } from "react";
import { useQuickCaptureAPI } from "@/hooks/useQuickCaptureAPI";
import { quickCaptureAPI } from "@/api/quickCaptureAPI";

const DRAFT_KEY = "quick-capture-draft";

export function QuickCaptureWindow() {
  const { appendToJournal } = useQuickCaptureAPI();
  const [text, setText] = useState(() => {
    // Restore draft on mount
    return localStorage.getItem(DRAFT_KEY) || "";
  });
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const closeWindow = async () => {
    console.log("[QuickCapture] Attempting to hide window via backend...");
    try {
      const response = await quickCaptureAPI.hide();
      if (response.success) {
        console.log("[QuickCapture] Window hidden successfully");
      } else {
        console.error("[QuickCapture] Failed to hide window:", response.error);
      }
    } catch (err) {
      console.error("[QuickCapture] Failed to hide window:", err);
    }
  };

  // Auto-focus immediately
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  // Close when the window loses focus
  useEffect(() => {
    const handleBlur = () => {
      console.log("[QuickCapture] Window blur detected, hiding");
      void closeWindow();
    };

    window.addEventListener("blur", handleBlur);
    return () => window.removeEventListener("blur", handleBlur);
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
    console.log("[QuickCapture] handleSubmit called with text:", text);
    const trimmedText = text.trim();
    if (!trimmedText) {
      console.log("[QuickCapture] No text to submit, returning");
      return;
    }

    // Clear the text state first
    setText("");

    // Clear draft from localStorage
    localStorage.removeItem(DRAFT_KEY);

    // Close window immediately
    await closeWindow();

    // Save in background - file watcher will auto-refresh main window
    console.log("[QuickCapture] Saving to journal:", trimmedText);
    appendToJournal(trimmedText).catch((err) => {
      // If save fails, restore draft so user doesn't lose content
      console.error("[QuickCapture] Save failed:", err);
      localStorage.setItem(DRAFT_KEY, trimmedText);
    });
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    console.log(
      "[QuickCapture] Key pressed:",
      e.key,
      "metaKey:",
      e.metaKey,
      "ctrlKey:",
      e.ctrlKey,
    );

    // Cmd/Ctrl+Enter to save
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      console.log(
        "[QuickCapture] Cmd/Ctrl+Enter detected, calling handleSubmit",
      );
      e.preventDefault();
      void handleSubmit();
    }
    if (e.key === "Escape") {
      console.log("[QuickCapture] Escape pressed, closing window");
      void closeWindow();
    }
  };

  // Detect dark mode
  const isDark =
    window.matchMedia &&
    window.matchMedia("(prefers-color-scheme: dark)").matches;

  // Colors from index.css - using popover colors for floating panel visibility
  // Dark: --popover: 0 0% 10% / 0.85, --popover-foreground: 0 0% 92%
  // Light: --popover: 0 0% 100%, --popover-foreground: 0 0% 12%
  const colors = isDark
    ? {
        background: "hsl(0 0% 10% / 0.5)",
        foreground: "hsl(0 0% 92%)",
        mutedForeground: "hsl(0 0% 65%)",
        border: "hsl(0 0% 50% / 0.2)",
        muted: "hsl(0 0% 30% / 0.5)",
      }
    : {
        background: "hsl(0 0% 100%)",
        foreground: "hsl(0 0% 12%)",
        mutedForeground: "hsl(0 0% 45%)",
        border: "hsl(0 0% 85%)",
        muted: "hsl(0 0% 94%)",
      };

  const containerStyle: React.CSSProperties = {
    height: "100vh",
    width: "100vw",
    padding: 8,
    backgroundColor: colors.background,
    borderRadius: 12,
  };

  const textareaStyle: React.CSSProperties = {
    width: "100%",
    height: "100%",
    padding: "12px 16px",
    fontSize: 14,
    fontFamily:
      "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    backgroundColor: "transparent",
    color: colors.foreground,
    borderRadius: 12,
    border: "none",
    outline: "none",
    resize: "none",
  };

  const placeholderColor = colors.mutedForeground;

  return (
    <div style={containerStyle}>
      <style>
        {`
          .quick-capture-textarea::placeholder {
            color: ${placeholderColor};
            opacity: 0.6;
            font-size: 12px;
          }
        `}
      </style>
      <textarea
        ref={textareaRef}
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="What's on your mind? (Cmd+Enter to save)"
        autoFocus
        rows={3}
        style={textareaStyle}
        className="quick-capture-textarea"
      />
    </div>
  );
}

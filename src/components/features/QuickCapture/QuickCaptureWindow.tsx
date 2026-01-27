/**
 * QuickCaptureWindow - Floating window for quick journal capture
 *
 * Optimized for speed: closes immediately on submit, save happens in background
 */

import React, { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQuickCaptureAPI } from "@/hooks/useQuickCaptureAPI";
import { quickCaptureAPI } from "@/api/quickCaptureAPI";

// Log to backend process
const backendLog = (message: string, level?: "info" | "warn" | "error" | "debug") => {
  invoke("log_from_frontend", { message, level }).catch(() => {});
};

const DRAFT_KEY = "quick-capture-draft";

export function QuickCaptureWindow() {
  const { appendToJournal } = useQuickCaptureAPI();
  const [text, setText] = useState(() => {
    // Restore draft on mount
    return localStorage.getItem(DRAFT_KEY) || "";
  });
  const [isFocused, setIsFocused] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Close guard to prevent duplicate close calls
  const isClosingRef = useRef(false);

  const closeWindow = async () => {
    // Prevent duplicate close calls
    if (isClosingRef.current) {
      backendLog("QuickCapture: Close already in progress, ignoring");
      return;
    }

    isClosingRef.current = true;
    backendLog("QuickCapture: Attempting to hide window");

    try {
      const response = await quickCaptureAPI.hide();
      if (response.success && response.data?.success) {
        backendLog(`QuickCapture: Window hidden, state: ${response.data.state}`);
      } else {
        backendLog(`QuickCapture: Failed to hide: ${response.data?.error || response.error}`, "error");
      }
    } catch (err) {
      backendLog(`QuickCapture: Hide error: ${err}`, "error");
    } finally {
      // Reset the guard after a short delay to allow for re-open
      setTimeout(() => {
        isClosingRef.current = false;
      }, 200);
    }
  };

  // Track window focus and auto-focus textarea
  useEffect(() => {
    const handleWindowFocus = () => {
      backendLog("QuickCapture: Window gained focus");
      setIsFocused(true);
      textareaRef.current?.focus();
    };

    const handleWindowBlur = () => {
      backendLog("QuickCapture: Window lost focus");
      setIsFocused(false);
    };

    window.addEventListener("focus", handleWindowFocus);
    window.addEventListener("blur", handleWindowBlur);

    // Check initial focus state
    if (document.hasFocus()) {
      backendLog("QuickCapture: Window has initial focus");
      setIsFocused(true);
      textareaRef.current?.focus();
    } else {
      backendLog("QuickCapture: Window does NOT have initial focus", "warn");
      setIsFocused(false);
    }

    return () => {
      window.removeEventListener("focus", handleWindowFocus);
      window.removeEventListener("blur", handleWindowBlur);
    };
  }, []);

  // Single unified keyboard handler with capture phase for reliability
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape to close (no text submission)
      if (e.key === "Escape") {
        backendLog("QuickCapture: Escape pressed, closing");
        e.preventDefault();
        e.stopPropagation();
        void closeWindow();
        return;
      }

      // Cmd/Ctrl+Enter to save and close
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        backendLog("QuickCapture: Cmd/Ctrl+Enter pressed, saving");
        e.preventDefault();
        e.stopPropagation();

        const trimmedText = text.trim();
        if (trimmedText) {
          // Clear the text state first
          setText("");
          // Clear draft from localStorage
          localStorage.removeItem(DRAFT_KEY);
          // Close window immediately
          void closeWindow();
          // Save in background
          appendToJournal(trimmedText).catch((err) => {
            backendLog(`QuickCapture: Save failed: ${err}`, "error");
            // If save fails, restore draft so user doesn't lose content
            localStorage.setItem(DRAFT_KEY, trimmedText);
          });
        }
      }
    };

    // Use capture phase to intercept events before they bubble
    window.addEventListener("keydown", handleKeyDown, { capture: true });
    return () =>
      window.removeEventListener("keydown", handleKeyDown, { capture: true });
  }, [text, appendToJournal]);

  // NOTE: Blur event handler removed intentionally
  // It caused accidental window closes during app switching (Cmd+Tab)
  // The window should only close via explicit user actions (Escape, Cmd+Enter)

  // Save draft on text change (debounced naturally by React state)
  useEffect(() => {
    if (text.trim()) {
      localStorage.setItem(DRAFT_KEY, text);
    } else {
      localStorage.removeItem(DRAFT_KEY);
    }
  }, [text]);

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
    border: isFocused ? "2px solid hsl(210 100% 50%)" : "2px solid transparent",
    boxSizing: "border-box",
    transition: "border-color 0.15s ease",
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
        placeholder="What's on your mind? (Cmd+Enter to save)"
        rows={3}
        style={textareaStyle}
        className="quick-capture-textarea"
      />
    </div>
  );
}
